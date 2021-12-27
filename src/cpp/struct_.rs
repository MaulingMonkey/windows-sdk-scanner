use crate::*;

use mmrbi::*;

use std::collections::*;
use std::fmt::{self, Debug, Formatter, Display};
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
    /// Location(s) this type was defined at.
    pub defined_at:             BTreeSet<Location>,
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

impl Display for AggregateCategory {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", match *self {
            AggregateCategory::Class        => "class",
            AggregateCategory::Struct       => "struct",
            AggregateCategory::Interface    => "interface",
            AggregateCategory::Union        => "union",
        })
    }
}

impl Aggregate {
    pub fn valid_name(name: &str) -> bool { valid_name(name) }

    pub fn new_class    (id: Ident) -> Self { Self::new(AggregateCategory::Class,  id) }
    pub fn new_struct   (id: Ident) -> Self { Self::new(AggregateCategory::Struct, id) }
    pub fn new_union    (id: Ident) -> Self { Self::new(AggregateCategory::Union,  id) }

    pub fn new(category: AggregateCategory, id: Ident) -> Self { Self { id, data: AggregateData { category, .. Default::default() }, defined_at: Default::default() } }

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
    pub(crate) fn add_from_cpp(&mut self, start: SrcToken, src: &mut SrcReader, typedef: bool) -> Result<(), ()> {
        let start_loc = src.token_to_location(start);
        self.defined_at.insert(start_loc.clone());
        self.data.add_from_cpp(start, src)?;

        macro_rules! expect_token { () => {
            src.next_token().ok_or_else(||{
                let msg = format!("expected `}}` to end `{} {}` before end of file", self.category, self.id);
                warning!(at: &start_loc.path, line: start_loc.line_no_or_0(), column: start_loc.col_no_or_0(), "{}", msg);
                self.issues.push(Issue::new(start_loc.clone(), msg));
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

        // given `typedef struct _ID { ... } ID;`, drop `_ID` in favor of `ID`
        let id_trim = self.id.trim_start_matches('_').trim_end_matches('_');
        for expr in exprs.into_iter() {
            if typedef && expr == id_trim {
                self.id = Ident::from(expr);
                break;
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
    pub(crate) fn add_from_cpp<'src>(&mut self, start: SrcToken, src: &'src mut SrcReader) -> Result<(), ()> {
        let category = self.category;

        macro_rules! issue {
            ( $token:expr, warn, $($tt:tt)* ) => {{
                let token = $token;
                let loc = src.token_to_location(token);
                let msg = format!($($tt)*);
                warning!(at: &loc.path, line: loc.line_no_or_0(), column: loc.col_no_or_0(), $($tt)*);
                self.issues.push(Issue::new(loc, msg));
            }};
            ( $token:expr, log, $($tt:tt)* ) => {{
                let token = $token;
                let loc = src.token_to_location(token);
                let msg = format!($($tt)*);
                self.issues.push(Issue::new(loc, msg));
            }};
        }

        macro_rules! expect_token { () => {
            src.next_token().ok_or_else(||{
                issue!(start, warn, "expected `}}` to end `{} {}` before end of file", category, start);
            })?
        }}

        'struct_: loop {
            let mut token = expect_token!();
            while token == "#" {
                let rest_of_line = src.next_line();
                let rest_of_line = rest_of_line.as_ref().map_or("", |l| &**l);
                issue!(token, log, "preprocessor command inside `{} {{ ... }}` not supported: #{}", self.category, rest_of_line);
                token = expect_token!();
            }

            let token_pos = src.position();
            match &*token {
                "}" => break 'struct_,
                "public" | "protected" | "private" => {
                    let colon = expect_token!();
                    if colon == ":" {
                        issue!(colon, log, "access specifiers (`{}:`) not yet supported", token);
                    } else {
                        issue!(colon, warn, "expected `:` after access specifier `{}`, got `{}` instead", token, colon);
                    }
                },
                "enum" => {
                    let name_or_brace = expect_token!();
                    let _brace = if name_or_brace == "{" { name_or_brace } else { expect_token!() };

                    let mut enum_ = EnumData::default();
                    let _ = enum_.add_from_cpp(token, src);

                    // field_name ;
                    let semi_or_name = expect_token!();
                    let (field_name, semi) = if semi_or_name == ";" {
                        (None, semi_or_name)
                    } else {
                        (Some(semi_or_name), expect_token!())
                    };

                    if semi == ";" {
                        let name = field_name.as_ref().map_or(Ident::from(""), |field_name| Ident::own(&**field_name));
                        self.fields.insert(name.clone(), Field::new_enum(enum_, name));
                        continue 'struct_
                    } else {
                        let field_name = field_name.as_ref().unwrap();
                        issue!(semi, warn, "expected `field_name ;` after sub-{}, instead got `{} {}`", token, field_name, semi);
                        while expect_token!() != ";" {}
                    }
                },
                "class" | "struct" | "interface" | "union" => {
                    //      struct      { ... }         field;
                    // or:  struct name { ... }         field;
                    // or:  struct name                 field;
                    // or:  struct name const * const   field;

                    let name_or_brace = expect_token!();
                    let agg_start = if name_or_brace == "{" {
                        // parsed:  struct {
                        Some(token)
                    } else {
                        // parsed:  struct name
                        let field_or_brace_or_morety = expect_token!();
                        if field_or_brace_or_morety == "{" {
                            // parsed:  struct name {
                            Some(name_or_brace)
                        } else {
                            // parsed:  struct name field
                            // or:      struct name const
                            // or:      struct name *
                            src.set_position(token_pos); // Awkwardly reset src pos before `struct` keyword
                            None
                        }
                    };

                    if let Some(agg_start) = agg_start {
                        // parsed:  struct name {
                        // or:      struct {
                        let mut agg = AggregateData::default();
                        agg.category = AggregateCategory::from_str(&*token).unwrap();
                        let _ = agg.add_from_cpp(agg_start, src);

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
                            issue!(semi, warn, "expected `field_name ;` after sub-{}, instead got `{} {}`", token, field_name, semi);
                            while expect_token!() != ";" {}
                        }
                    }
                },
                _ => {},
            }

            // parse:   const int * const   name;
            // as:      ty    ty  ty ty     name;
            let mut ty = String::new();
            let mut possible_name = token;
            let mut braces = 0;
            loop {
                let token = expect_token!();
                match &*token {
                    ";" if braces == 0 => {
                        let name = Ident::own(&*possible_name);
                        self.fields.insert(name.clone(), Field::new(ty, name));
                        continue 'struct_
                    },
                    ":" => {
                        let ty = Ident::from(ty);
                        let name = Ident::own(&*possible_name);
                        let f = self.fields.entry(name.clone()).or_insert_with(move || Field::new(ty, name));
                        let bits = expect_token!();
                        f.bits = bits.parse::<u32>().map_or(None, NonZeroU32::new);
                        if f.bits.is_none() { issue!(bits, warn, "{} {} contains invalid bitset `: {}`", self.category, start, bits); }
                        while expect_token!() != ";" {}
                        continue 'struct_
                    },
                    // "enum" => { ... },
                    // "class" | "struct" | "interface" | "union" => { ... },
                    "{" => {
                        if !ty.is_empty() { ty.push(' ') }
                        ty.push_str(&*possible_name); // wasn't a name
                        possible_name = token;
                        braces += 1;
                    },
                    "}" => {
                        if braces > 0 {
                            if !ty.is_empty() { ty.push(' ') }
                            ty.push_str(&*possible_name); // wasn't a name
                            possible_name = token;
                            braces -= 1;
                        } else {
                            issue!(token, warn, "expected `field_name ;` before end of `{} {}`", self.category, start);
                            break 'struct_
                        }
                    },
                    _ => {
                        if !ty.is_empty() { ty.push(' ') }
                        ty.push_str(&*possible_name); // wasn't a name
                        possible_name = token;
                    },
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
