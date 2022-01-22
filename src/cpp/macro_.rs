use crate::*;

use std::collections::BTreeSet;
use std::fmt::{self, Debug, Formatter};



/// `#define ID(...) ...`
#[derive(Clone)]
pub struct Macro {
    /// Location(s) this macro was defined at.
    pub defined_at:             BTreeSet<Location>,

    /// Identifier of this macro (e.g. `ID`)
    pub id:                     Ident,

    pub(crate) _non_exhaustive: (),
}

impl Macro {
    pub fn new(id: impl Into<Ident>) -> Self {
        Self {
            defined_at: Default::default(),
            id: id.into(),
            _non_exhaustive: Default::default(),
        }
    }
}

impl Debug for Macro {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Macro {{ id: {:?}, .. }}", self.id)
    }
}
