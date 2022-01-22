use crate::*;

use std::fmt::{self, Debug, Formatter};



/// `#define ID(...) ...`
#[derive(Clone)]
pub struct Macro {
    pub id:                     Ident,
    pub(crate) _non_exhaustive: (),
}

impl Debug for Macro {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Macro {{ id: {:?}, .. }}", self.id)
    }
}
