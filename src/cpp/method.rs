use crate::*;

use std::fmt::{self, Debug, Formatter};
use std::sync::atomic::{AtomicBool, Ordering};



/// `void id();` belonging to an [`Interface`], class, or structure.
pub struct Method {
    /// The type this method belongs to, such as `IUnknown`.
    pub ty:                     Ident,

    /// The function data - e.g. `f.id` might be `AddRef`.
    pub f:                      Function,

    /// The method is inherited from a base class.
    pub(crate) inherited:       AtomicBool,

    pub(crate) _non_exhaustive: (),
}

impl Method {
    pub fn valid_name(name: &str) -> bool { valid_name(name) }
    pub fn new(ty: Ident, id: Ident) -> Self { Self { ty, f: Function::new(id), inherited: AtomicBool::new(false), _non_exhaustive: () } }
    pub(crate) fn is_inherited(&self) -> bool { self.inherited.load(Ordering::Relaxed) }
}

impl Debug for Method {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "Method {{ ty: {:?}, f.id: {:?}, inherited: {}, ... }}", self.ty, self.f.id, self.inherited.load(Ordering::Relaxed))
    }
}
