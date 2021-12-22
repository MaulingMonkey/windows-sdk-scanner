use crate::*;

use std::fmt::{self, Debug, Formatter};



/// `struct { ty id; };`
pub struct Field {
    pub ty:                     Ident,
    pub id:                     Ident,
    pub(crate) _non_exhaustive: (),
}

impl Debug for Field {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "Field {{ ty: {:?}, id: {:?}, ... }}", self.ty, self.id)
    }
}
