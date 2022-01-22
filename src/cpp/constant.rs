use crate::*;

use std::collections::BTreeSet;
use std::fmt::{self, Debug, Formatter};



/// `#define FOO 1` or `const uint32_t FOO = 1;`
pub struct Constant {
    /// Location(s) this constant was defined at.
    pub defined_at:             BTreeSet<Location>,

    /// Identifier of this constant (e.g. `FOO`)
    pub id:                     Ident,

    pub(crate) _non_exhaustive: (),
}

impl Constant {
    pub fn new(id: impl Into<Ident>) -> Self {
        Self {
            defined_at: Default::default(),
            id: id.into(),
            _non_exhaustive: Default::default(),
        }
    }
}

impl Debug for Constant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Constant {{ id: {:?}, .. }}", self.id)
    }
}
