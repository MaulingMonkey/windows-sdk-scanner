use crate::*;



/// `namespace foo { ... }`
#[derive(Debug)]
pub struct Namespace {
    pub id:                     Ident,
    pub(crate) _non_exhaustive: (),
}
