use crate::*;

use std::collections::BTreeSet;
use std::fmt::{self, Debug, Formatter};



/// `void id();`
pub struct Function {
    /// Location(s) this function was defined at.
    pub defined_at:             BTreeSet<Location>,

    /// The function name - e.g. might be `AddRef` or `Direct3DCreate9`.
    pub id:                     Ident,

    pub abi:                    FunctionAbi,
    // args
    // return type

    pub(crate) _non_exhaustive: (),
}

/// `WINAPI`, `stdcall`, etc.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FunctionAbi {
    Default,
    Winapi,
    Stdcall,
}



impl Function {
    pub fn valid_name(name: &str) -> bool { valid_name(name) }
    pub fn new(id: Ident) -> Self { Self { defined_at: Default::default(), id, abi: FunctionAbi::Default, _non_exhaustive: () } }
}

impl Debug for Function {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "Function {{ id: {:?}, ... }}", self.id)
    }
}
