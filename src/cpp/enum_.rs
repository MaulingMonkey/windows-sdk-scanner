use crate::*;

use mmrbi::*;

use std::fmt::{self, Debug, Formatter};



/// `typedef enum { ... } id;`
pub struct Enum {
    pub id:                     Ident,
    pub data:                   EnumData,
}

#[derive(Default)]
pub struct EnumData {
    pub values:                 VecMap<Ident, Option<String>>,
    pub issues:                 Vec<Issue>,
    pub(crate) _non_exhaustive: (),
}

impl Enum {
    pub fn valid_name(name: &str) -> bool { valid_name(name) }

    pub fn new(id: Ident) -> Self { Self { id, data: Default::default() } }

    pub(crate) fn add_from_cpp(&mut self, enum_start: &Location, src: &mut SrcReader, typedef: bool) -> Result<(), ()> {
        macro_rules! err {
            ( $($tt:tt)* ) => {
                warning!(at: &enum_start.path, line: enum_start.line_no_or_0(), $($tt)*)
            };
        }

        macro_rules! expect_token { () => {
            src.next_token().ok_or_else(||{
                self.data.issues.push(Issue::new(enum_start.clone(), "expected `}}` to end enum before end of file"));
                err!("expected `}}` to end enum before end of file")
            })?
        }}

        'enum_: loop {
            let mut token = expect_token!();
            while token == "#" {
                let rest_of_line = src.next_line();
                let rest_of_line = rest_of_line.as_ref().map_or("", |l| &**l);
                self.data.issues.push(Issue::new(
                    src.token_to_location(token),
                    format!("preprocessor command inside `enum {{ ... }}` not supported: #{}", rest_of_line)
                ));
                token = expect_token!();
            }

            if token == "}" { break 'enum_ }

            let value = self.data.values.entry(Ident::own(&*token)).or_insert_with(|| None);
            let token = expect_token!();
            match &*token {
                "=" => {
                    'value: loop {
                        let token = expect_token!();
                        match &*token {
                            "#" => { // probably a preprocessor command
                                let rest_of_line = src.next_line();
                                let rest_of_line = rest_of_line.as_ref().map_or("", |l| &**l);
                                self.data.issues.push(Issue::new(
                                    src.token_to_location(token),
                                    format!("preprocessor command inside `enum {{ ... }}` not supported: #{}", rest_of_line)
                                ));
                            },
                            "," => break 'value,
                            "}" => break 'enum_,
                            more => {
                                if let Some(value) = value {
                                    value.push(' ');
                                    value.push_str(more);
                                } else {
                                    *value = Some(more.into());
                                }
                            },
                        }
                    }
                },
                "#" => { // probably a preprocessor command
                    let rest_of_line = src.next_line();
                    let rest_of_line = rest_of_line.as_ref().map_or("", |l| &**l);
                    self.data.issues.push(Issue::new(
                        src.token_to_location(token),
                        format!("preprocessor command inside `enum {{ ... }}` not supported: #{}", rest_of_line)
                    ));
                },
                ","     => continue 'enum_,
                "}"     => break 'enum_,
                other   => drop(err!("expected `= value,` or `,` after enumerand name, instead got `{}`", other)),
            }
        }
        // TODO: try to parse `name, name, name;` first?
        Ok(())
    }
}

impl Debug for Enum {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("Enum")
            .field("id",            &self.id            )
            .field("values",        &self.data.values   )
            .field("issues",        &self.data.issues   )
            .finish_non_exhaustive()
    }
}

impl Debug for EnumData {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("EnumData")
            .field("values",        &self.values        )
            .field("issues",        &self.issues        )
            .finish_non_exhaustive()
    }
}
