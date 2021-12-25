use crate::*;

use mmrbi::*;

use std::fmt::{self, Debug, Formatter};
use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};



pub type Class      = Aggregate;
pub type Struct     = Aggregate;
pub type Union      = Aggregate;

pub type ClassData  = AggregateData;
pub type StructData = AggregateData;
pub type UnionData  = AggregateData;

/// `typedef [class | struct | union] _id { ... } id;`
pub struct Aggregate {
    pub id:                     Ident,
    pub data:                   StructData,
    // typedefs?
}

/// `[class | struct | union] { ... }`
#[derive(Default)]
pub struct AggregateData {
    pub category:               AggregateCategory,
    pub base:                   Option<Ident>,
    pub fields:                 VecMap<Ident, Field>,
    pub issues:                 Vec<Issue>,
    pub(crate) _non_exhaustive: (),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AggregateCategory {
    Class,
    Struct,
    Interface,
    Union,
}

impl Default for AggregateCategory { fn default() -> Self { AggregateCategory::Struct } }

impl AggregateCategory {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "class"     => AggregateCategory::Class,
            "struct"    => AggregateCategory::Struct,
            "interface" => AggregateCategory::Interface,
            "union"     => AggregateCategory::Union,
            _           => return None,
        })
    }
}

impl Aggregate {
    pub fn valid_name(name: &str) -> bool { valid_name(name) }

    pub fn new_class    (id: Ident) -> Self { Self::new(AggregateCategory::Class,  id) }
    pub fn new_struct   (id: Ident) -> Self { Self::new(AggregateCategory::Struct, id) }
    pub fn new_union    (id: Ident) -> Self { Self::new(AggregateCategory::Union,  id) }

    pub fn new(category: AggregateCategory, id: Ident) -> Self { Self { id, data: AggregateData { category, .. Default::default() } } }

    /// Parse e.g. `ty1 name1; ty2 name2; }`
    ///
    /// Expects you've already parsed:
    /// *   Any initial `typedef` keyword
    /// *   The initial `struct` keyword
    /// *   Any initial struct name
    /// *   The initial opening brace `{`
    ///
    /// Parses:
    /// *   Struct field type/name pairs
    /// *   The closing brace `}` of the struct
    /// *   Any trailing names for struct typedefs or instances
    /// *   The closing `;` of the struct
    ///
    pub(crate) fn add_from_cpp(&mut self, start: &Location, src: &mut SrcReader, typedef: bool) -> Result<(), ()> {
        self.data.add_from_cpp(start, src)?;

        macro_rules! err {
            ( $($tt:tt)* ) => {
                warning!(at: &start.path, line: start.line_no_or_0(), $($tt)*)
            };
        }

        macro_rules! expect_token { () => {
            src.next_token().ok_or_else(||{
                self.issues.push(Issue::new(start.clone(), "expected `}}` to end enum before end of file"));
                err!("expected `}}` to end enum before end of file")
            })?
        }}

        let mut expr = String::new();
        let mut exprs = Vec::new();
        loop {
            let token = expect_token!();
            match &*token {
                ";" => break,
                "," => exprs.push(std::mem::take(&mut expr)),
                s => {
                    if !expr.is_empty() { expr.push(' ') }
                    expr.push_str(s);
                }
            }
        }
        if !expr.is_empty() { exprs.push(expr) }

        for expr in exprs.into_iter() {
            // given `typedef struct _ID { ... } ID;`, drop `_ID` in favor of `ID`
            if typedef && self.id.starts_with("_") && self.id[1..] == expr {
                self.id = Ident::from(expr);
            }
        }

        Ok(())
    }
}

impl AggregateData {
    /// Parse e.g. `ty1 name1; ty2 name2; }`
    ///
    /// Expects you've already parsed:
    /// *   Any initial `typedef` keyword
    /// *   The initial `struct` keyword
    /// *   Any initial struct name
    /// *   The initial opening brace `{`
    ///
    /// Parses:
    /// *   Struct field type/name pairs
    /// *   The closing brace `}` of the struct
    ///
    /// Does *not* parse:
    /// *   Any trailing names for struct typedefs or instances
    /// *   The closing `;` of the struct
    ///
    pub(crate) fn add_from_cpp<'src>(&mut self, start: &Location, src: &'src mut SrcReader) -> Result<(), ()> {
        macro_rules! err {
            ( $($tt:tt)* ) => {
                warning!(at: &start.path, line: start.line_no_or_0(), $($tt)*)
            };
        }

        macro_rules! expect_token { () => {
            src.next_token().ok_or_else(||{
                self.issues.push(Issue::new(start.clone(), "expected `}}` to end enum before end of file"));
                err!("expected `}}` to end enum before end of file")
            })?
        }}

        'struct_: loop {
            let mut token = expect_token!();
            while token == "#" {
                let rest_of_line = src.next_line();
                let rest_of_line = rest_of_line.as_ref().map_or("", |l| &**l);
                self.issues.push(Issue::new(
                    src.token_to_location(token),
                    format!("preprocessor command inside `struct {{ ... }}` not supported: #{}", rest_of_line)
                ));
                token = expect_token!();
            }

            match &*token {
                "}" => break 'struct_,
                "enum" | "struct" | "union" => {
                    let name_or_brace = expect_token!();
                    if name_or_brace != "{" {
                        let _name = expect_token!();
                    }

                    let mut agg = AggregateData::default();
                    agg.category = AggregateCategory::from_str(&*token).unwrap();
                    let _ = agg.add_from_cpp(&src.token_to_location(token), src);

                    // field_name ;
                    let semi_or_name = expect_token!();
                    let (field_name, semi) = if semi_or_name == ";" {
                        (None, semi_or_name)
                    } else {
                        (Some(semi_or_name), expect_token!())
                    };

                    if semi == ";" {
                        let name = field_name.as_ref().map_or(Ident::from(""), |field_name| Ident::own(&**field_name));
                        self.fields.insert(name.clone(), Field::new_agg(agg, name));
                        continue 'struct_
                    } else {
                        let field_name = field_name.as_ref().unwrap();
                        let loc = src.token_to_location(semi);
                        warning!(at: &start.path, line: loc.line_no_or_0(), "expected `field_name ;` after sub-{}, instead got `{} {}`", token, field_name, semi);
                        self.issues.push(Issue::new(loc, format!("expected `field_name ;` after sub-{}, instead got `{} {}`", token, field_name, semi)));
                        while expect_token!() != ";" {}
                    }
                },
                _ => {},
            }

            // parse:   const int * const   name;
            // as:      ty    ty  ty ty     name;
            let mut ty = String::new();
            let mut possible_name = token;
            loop {
                let token = expect_token!();
                if token == ";" {
                    let name = Ident::own(&*possible_name);
                    self.fields.insert(name.clone(), Field::new(ty, name));
                    continue 'struct_
                } else if token == ":" {
                    let ty = Ident::from(ty);
                    let name = Ident::own(&*possible_name);
                    let f = self.fields.entry(name.clone()).or_insert_with(move || Field::new(ty, name));
                    f.bits = expect_token!().parse::<u32>().map_or(None, NonZeroU32::new);
                    if f.bits.is_none() { err!("struct contains invalid bitsets"); }
                    while expect_token!() != ";" {}
                    continue 'struct_
                } else {
                    if !ty.is_empty() { ty.push(' ') }
                    ty.push_str(&*possible_name);
                    possible_name = token;
                }
            }
        }

        Ok(())
    }
}

impl Debug for Aggregate {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("Aggregate")
            .field("id",            &self.id                                    )
            .field("category",      &self.category                              )
            .field("base",          &self.base                                  )
            .field("fields",        &self.fields.values_by_insert().collect::<Vec<_>>()   )
            .field("issues",        &self.issues                                )
            .finish_non_exhaustive()
    }
}

impl Debug for AggregateData {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("AggregateData")
            .field("category",      &self.category                              )
            .field("base",          &self.base                                  )
            .field("fields",        &self.fields.values_by_insert().collect::<Vec<_>>()   )
            .field("issues",        &self.issues                                )
            .finish_non_exhaustive()
    }
}

impl Deref for Aggregate {
    type Target = AggregateData;
    fn deref(&self) -> &Self::Target { &self.data }
}

impl DerefMut for Aggregate {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.data }
}
