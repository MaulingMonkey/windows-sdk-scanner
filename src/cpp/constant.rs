use crate::*;

use std::fmt::{self, Debug, Formatter};



/// `#define FOO 1` or `const uint32_t FOO = 1;`
pub struct Constant {
    pub id:                     Ident,
    pub(crate) _non_exhaustive: (),
}

impl Debug for Constant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Constant {{ id: {:?}, .. }}", self.id)
    }
}
