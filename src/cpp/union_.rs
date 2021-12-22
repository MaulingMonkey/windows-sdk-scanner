use crate::*;

use mmrbi::*;

use std::fmt::{self, Debug, Formatter};
use std::io;
use std::ops::{Deref, DerefMut};



/// `typedef union { ... } id;`
pub struct Union {
    pub id:                     Ident,
    pub data:                   UnionData,
    // typedefs?
}

#[derive(Default)]
pub struct UnionData {
    pub base:                   Option<Ident>,
    pub fields:                 VecMap<Ident, Field>,
    pub(crate) _non_exhaustive: (),
}

impl Union {
    pub fn valid_name(name: &str) -> bool { valid_name(name) }
    pub fn new(id: Ident) -> Self { Self { id, data: Default::default() } }

    pub(crate) fn add_from_cpp(&mut self, start: &Location, src: &mut SrcReader, typedef: bool) -> io::Result<()> {
        for def in self.data.add_from_cpp(start, src)? {
            if !typedef             { continue }
            if def.starts_with("*") { continue }
            if self.id.as_str().starts_with("_") && &self.id.as_str()[1..] == def {
                self.id = Ident::own(def);
            } else if self.id.as_str() != def {
                // TODO: add typedef/alias
            }
        }
        Ok(())
    }
}

impl UnionData {
    pub(crate) fn add_from_cpp<'src>(&mut self, start: &Location, src: &'src mut SrcReader) -> io::Result<impl Iterator<Item = &'src str>> {
        while let Some(SrcLine { location, raw, trimmed }) = src.next_line() {
            if trimmed.is_empty() { continue }
            if trimmed.starts_with("#") { continue } // preprocessor spam
            if trimmed.starts_with("//") { continue } // C++ single line comment
            if trimmed.starts_with("/*") && trimmed.ends_with("*/") { continue } // C single line comment

            if let Some(defs) = trimmed.strip_prefix_suffix("}", ";") {
                let defs = defs.trim();
                return Ok((!defs.is_empty()).then(|| defs.split(',').map(str::trim)).into_iter().flatten());
            }

            if let Some((ty, rest)) = trimmed.split_once_trim(" ") {
                if let Some((field, _rest)) = rest.split_once_trim(";") {
                    self.add_field(&location, Ident::own(ty), Ident::own(field));
                    continue
                }
            }
            warning!(at: &start.path, line: start.line_no_or_0(), "unexpected line `{}` in `union` definition", raw);
            // TODO: handle sub-`union {` and sub-`struct {`s
        }
        Err(unexpected_eof(&start, "end of union via `};`"))
    }

    fn add_field(&mut self, _location: &Location, ty: Ident, name: Ident) {
        self.fields.insert(name.clone(), Field { ty, id: name, _non_exhaustive: () });
    }
}

impl Debug for Union {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("Union")
            .field("id",            &self.id                                    )
            .field("base",          &self.base                                  )
            .field("fields",        &self.fields.values_by_insert().collect::<Vec<_>>()   )
            .finish_non_exhaustive()
    }
}

impl Debug for UnionData {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("UnionData")
            .field("base",          &self.base                                  )
            .field("fields",        &self.fields.values_by_insert().collect::<Vec<_>>()   )
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
