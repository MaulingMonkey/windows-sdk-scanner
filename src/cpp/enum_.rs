use crate::*;

use mmrbi::*;

use std::collections::*;
use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};



/// `typedef enum _ID { ... } ID;` or
/// `typedef enum class _ID { ... } ID;`
pub struct Enum {
    pub id:                     Ident,
    pub data:                   EnumData,
    /// Location(s) this type was defined at.
    pub defined_at:             BTreeSet<Location>,
}

#[derive(Default)]
pub struct EnumData {
    pub class:                  bool,
    pub values:                 VecMap<Ident, Option<String>>,
    pub issues:                 Vec<Issue>,
    pub(crate) _non_exhaustive: (),
}

impl Enum {
    pub fn valid_name(name: &str) -> bool { valid_name(name) }

    pub fn new(id: Ident) -> Self { Self { id, data: Default::default(), defined_at: Default::default() } }

    /// Parse e.g. `name1, name2 = name2 } alias1, alias2 ;`
    ///
    /// Expects you've already parsed:
    /// *   Any initial `typedef` keyword
    /// *   The initial `enum` or `enum class` keyword(s)
    /// *   Any initial enum name
    /// *   The initial opening brace `{`
    ///
    /// Parses:
    /// *   Enum key/value pairs
    /// *   The closing brace `}` of the enum
    /// *   Any trailing names for enum typedefs or instances
    /// *   The closing `;` of the enum
    ///
    pub(crate) fn add_from_cpp(&mut self, start: SrcToken, src: &mut SrcReader, typedef: bool) -> Result<(), ()> {
        let start_loc = src.token_to_location(start);
        self.defined_at.insert(start_loc.clone());
        self.data.add_from_cpp(start, src)?;

        macro_rules! expect_token { () => {
            src.next_token().ok_or_else(||{
                let msg = "expected `}}` to end enum before end of file";
                warning!(at: &start_loc.path, line: start_loc.line_no_or_0(), column: start_loc.col_no_or_0(), "{}", msg);
                self.issues.push(Issue::new(start_loc.clone(), "expected `}}` to end enum before end of file"));
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

impl EnumData {
    /// Parse e.g. `name1, name2 = name2 }`
    ///
    /// Expects you've already parsed:
    /// *   Any initial `typedef` keyword
    /// *   The initial `enum` or `enum class` keyword(s)
    /// *   Any initial enum name
    /// *   The initial opening brace `{`
    ///
    /// Parses:
    /// *   Enum key/value pairs
    /// *   The closing brace `}` of the enum
    ///
    /// Does *not* parse:
    /// *   Any trailing names for enum typedefs or instances
    /// *   The closing `;` of the enum
    ///
    pub(crate) fn add_from_cpp(&mut self, start: SrcToken, src: &mut SrcReader) -> Result<(), ()> {
        let start = src.token_to_location(start);

        macro_rules! expect_token { () => {
            src.next_token().ok_or_else(||{
                let msg = "expected `}}` to end enum before end of file";
                warning!(at: &start.path, line: start.line_no_or_0(), column: start.col_no_or_0(), "{}", msg);
                self.issues.push(Issue::new(start.clone(), msg));
            })?
        }}

        let mut warn_enumerand_name = false;

        'enum_: loop {
            let mut token = expect_token!();
            while token == "#" {
                let rest_of_line = src.next_line();
                let rest_of_line = rest_of_line.as_ref().map_or("", |l| &**l);
                self.issues.push(Issue::new(
                    src.token_to_location(token),
                    format!("preprocessor command inside `enum {{ ... }}` not supported: #{}", rest_of_line)
                ));
                token = expect_token!();
            }

            if token == "}" { break 'enum_ }

            let value = self.values.entry(Ident::own(&*token)).or_insert_with(|| None);
            let token = expect_token!();
            match &*token {
                "=" => {
                    let value = value.get_or_insert(Default::default());
                    let mut parens = 0;
                    'value: loop {
                        let token = expect_token!();
                        match &*token {
                            "#" => { // probably a preprocessor command
                                let rest_of_line = src.next_line();
                                let rest_of_line = rest_of_line.as_ref().map_or("", |l| &**l);
                                self.issues.push(Issue::new(
                                    src.token_to_location(token),
                                    format!("preprocessor command inside `enum {{ ... }}` not supported: #{}", rest_of_line)
                                ));
                            },
                            "," if parens == 0 => break 'value,
                            "}" => break 'enum_,
                            ")" if parens == 0 => {
                                let msg = "more closing `)` parens than opening `(` parens in enumerand value";
                                let loc = src.token_to_location(token);
                                error!(at: &loc.path, line: loc.line_no_or_0(), column: loc.col_no_or_0(), "{}", msg);
                                self.issues.push(Issue::new(loc, msg));
                                value.push_str(" )");
                            },
                            "(" => { parens += 1; value.push_str("("); }, // avoid initial whitespace
                            ")" => { parens -= 1; value.push_str(" )"); },
                            "," | "<" | "{" | "[" => value.push_str(&*token), // avoid initial whitespace
                            more => {
                                if !value.is_empty() { value.push(' ') }
                                value.push_str(more);
                            },
                        }
                    }
                },
                "#" => { // probably a preprocessor command
                    let rest_of_line = src.next_line();
                    let rest_of_line = rest_of_line.as_ref().map_or("", |l| &**l);
                    self.issues.push(Issue::new(
                        src.token_to_location(token),
                        format!("preprocessor command inside `enum {{ ... }}` not supported: #{}", rest_of_line)
                    ));
                },
                ","     => continue 'enum_,
                "}"     => break 'enum_,
                _       => {
                    if !warn_enumerand_name {
                        warn_enumerand_name = true;
                        let msg = format!("expected `= value,` or `,` after enumerand name, instead got `{}`", token);
                        let loc = src.token_to_location(token);
                        warning!(at: &loc.path, line: loc.line_no_or_0(), column: loc.col_no_or_0(), "{}", msg);
                        self.issues.push(Issue::new(loc, &msg));
                    }
                    let _ = src.next_line();
                },
            }
        }
        // TODO: try to parse `name, name, name;` first?
        Ok(())
    }
}

impl Debug for Enum {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("Enum")
            .field("id",            &self.id        )
            .field("class",         &self.class     )
            .field("values",        &self.values    )
            .field("issues",        &self.issues    )
            .finish_non_exhaustive()
    }
}

impl Debug for EnumData {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("EnumData")
            .field("class",         &self.class         )
            .field("values",        &self.values        )
            .field("issues",        &self.issues        )
            .finish_non_exhaustive()
    }
}

impl Deref for Enum {
    type Target = EnumData;
    fn deref(&self) -> &Self::Target { &self.data }
}

impl DerefMut for Enum {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.data }
}
