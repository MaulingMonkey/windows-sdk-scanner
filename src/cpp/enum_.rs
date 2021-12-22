use crate::*;

use std::io;



/// `typedef enum { ... } id;`
#[derive(Debug)]
pub struct Enum {
    pub id:                     Ident,
    pub(crate) _non_exhaustive: (),
}

impl Enum {
    pub fn valid_name(name: &str) -> bool { valid_name(name) }

    pub fn new(id: Ident) -> Self { Self { id, _non_exhaustive: () } }

    pub(crate) fn add_from_cpp(&mut self, enum_start: &Location, src: &mut SrcReader, typedef: bool) -> io::Result<()> {
        while let Some(line) = src.next_line() {
            if let Some(defs) = line.trimmed.strip_prefix_suffix("}", ";") {
                // TODO: ...typedefs/instances...
                let _ = defs;
                let _ = typedef;
                return Ok(())
            }
        }
        Err(unexpected_eof(&enum_start, "end of enum via `};`"))
    }

    fn _add_value(&mut self, _location: &Location, _value: ()) {
        // TODO: add value
    }
}
