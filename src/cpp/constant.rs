use crate::*;



/// `#define FOO 1` or `const uint32_t FOO = 1;`
#[derive(Debug)]
pub struct Constant {
    pub id:                     Ident,
    pub(crate) _non_exhaustive: (),
}
