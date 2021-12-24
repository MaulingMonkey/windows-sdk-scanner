use crate::*;

use mmrbi::*;

use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};



/// `typedef struct _id { ... } id;`
pub struct Struct {
    pub id:                     Ident,
    pub data:                   StructData,
    // typedefs?
}

/// `struct { ... }`
#[derive(Default)]
pub struct StructData {
    pub base:                   Option<Ident>,
    pub fields:                 VecMap<Ident, Field>,
    pub issues:                 Vec<Issue>,
    pub(crate) _non_exhaustive: (),
}

impl Struct {
    pub fn valid_name(name: &str) -> bool { valid_name(name) }
    pub fn new(id: Ident) -> Self { Self { id, data: Default::default() } }

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

impl StructData {
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

        let mut warned_subtype = false;

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
                    if !warned_subtype {
                        warned_subtype = true;
                        let loc = src.token_to_location(token);
                        warning!(at: &loc.path, line: loc.line_no_or_0(), "(anonymous?) sub-`{}` in `struct` not yet supported", token);
                        self.issues.push(Issue::new(loc, format!("(anonymous?) sub-`{}` in `struct` not yet supported", token)));
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
                    self.add_field(src.token_to_location(possible_name), Ident::from(ty), Ident::own(&*possible_name));
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

    fn add_field(&mut self, _location: Location, ty: Ident, name: Ident) {
        self.fields.insert(name.clone(), Field { ty, id: name, _non_exhaustive: () });
    }
}

impl Debug for Struct {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("Struct")
            .field("id",            &self.id                                    )
            .field("base",          &self.base                                  )
            .field("fields",        &self.fields.values_by_insert().collect::<Vec<_>>()   )
            .field("issues",        &self.issues                                )
            .finish_non_exhaustive()
    }
}

impl Debug for StructData {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("StructData")
            .field("base",          &self.base                                  )
            .field("fields",        &self.fields.values_by_insert().collect::<Vec<_>>()   )
            .field("issues",        &self.issues                                )
            .finish_non_exhaustive()
    }
}

impl Deref for Struct {
    type Target = StructData;
    fn deref(&self) -> &Self::Target { &self.data }
}

impl DerefMut for Struct {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.data }
}
