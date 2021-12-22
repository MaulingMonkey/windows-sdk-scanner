use crate::*;



/// `typedef enum { ... } id;`
#[derive(Debug)]
pub struct Flags {
    pub id:                     Ident,
    pub(crate) _non_exhaustive: (),
}
