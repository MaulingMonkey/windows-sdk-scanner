use crate::*;

use mmrbi::*;

use std::fmt::{self, Debug, Formatter};
use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};



/// `typedef union _id { ... } id;`
pub struct Union {
    pub id:                     Ident,
    pub data:                   UnionData,
    // typedefs?
}

/// `union { ... }`
#[derive(Default)]
pub struct UnionData {
    pub base:                   Option<Ident>,
    pub fields:                 VecMap<Ident, Field>,
    pub issues:                 Vec<Issue>,
    pub(crate) _non_exhaustive: (),
}

impl Union {
    pub fn valid_name(name: &str) -> bool { valid_name(name) }
    pub fn new(id: Ident) -> Self { Self { id, data: Default::default() } }

    /// Parse e.g. `ty1 name1; ty2 name2; }`
    ///
    /// Expects you've already parsed:
    /// *   Any initial `typedef` keyword
    /// *   The initial `union` keyword
    /// *   Any initial union name
    /// *   The initial opening brace `{`
    ///
    /// Parses:
    /// *   Union field type/name pairs
    /// *   The closing brace `}` of the union
    /// *   Any trailing names for union typedefs or instances
    /// *   The closing `;` of the union
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
            // given `typedef union _ID { ... } ID;`, drop `_ID` in favor of `ID`
            if typedef && self.id.starts_with("_") && self.id[1..] == expr {
                self.id = Ident::from(expr);
            }
        }

        Ok(())
    }
}

impl UnionData {
    /// Parse e.g. `ty1 name1; ty2 name2; }`
    ///
    /// Expects you've already parsed:
    /// *   Any initial `typedef` keyword
    /// *   The initial `union` keyword
    /// *   Any initial union name
    /// *   The initial opening brace `{`
    ///
    /// Parses:
    /// *   Union field type/name pairs
    /// *   The closing brace `}` of the union
    ///
    /// Does *not* parse:
    /// *   Any trailing names for union typedefs or instances
    /// *   The closing `;` of the union
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

        let mut warned_subtype = false;

        'struct_: loop {
            let mut token = expect_token!();
            while token == "#" {
                let rest_of_line = src.next_line();
                let rest_of_line = rest_of_line.as_ref().map_or("", |l| &**l);
                self.issues.push(Issue::new(
                    src.token_to_location(token),
                    format!("preprocessor command inside `union {{ ... }}` not supported: #{}", rest_of_line)
                ));
                token = expect_token!();
            }

            match &*token {
                "}" => break 'struct_,
                "enum" | "struct" | "union" => {
                    if !warned_subtype {
                        warned_subtype = true;
                        let loc = src.token_to_location(token);
                        warning!(at: &loc.path, line: loc.line_no_or_0(), "(anonymous?) sub-`{}` in `union` not yet supported", token);
                        self.issues.push(Issue::new(loc, format!("(anonymous?) sub-`{}` in `union` not yet supported", token)));
                    }
                    // continue trying to parse what we can anyways
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
                    if f.bits.is_none() { err!("union contains invalid bitsets"); }
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

impl Debug for Union {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("Union")
            .field("id",            &self.id                                    )
            .field("base",          &self.base                                  )
            .field("fields",        &self.fields.values_by_insert().collect::<Vec<_>>()   )
            .field("issues",        &self.issues                                )
            .finish_non_exhaustive()
    }
}

impl Debug for UnionData {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("UnionData")
            .field("base",          &self.base                                  )
            .field("fields",        &self.fields.values_by_insert().collect::<Vec<_>>()   )
            .field("issues",        &self.issues                                )
            .finish_non_exhaustive()
    }
}

impl Deref for Union {
    type Target = UnionData;
    fn deref(&self) -> &Self::Target { &self.data }
}

impl DerefMut for Union {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.data }
}
