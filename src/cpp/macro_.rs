use crate::*;



/// `#define ID(...) ...`
#[derive(Debug)]
pub struct Macro {
    pub id:                     Ident,
    pub(crate) _non_exhaustive: (),
}
