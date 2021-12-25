use crate::*;

use std::fmt::{self, Debug, Formatter};
use std::num::NonZeroU32;



/// `ty id;` as found inside a `struct` or `union`.
pub struct Field {
    pub ty:     Type,
    pub id:     Ident,
    pub bits:   Option<NonZeroU32>,
    _ne:        (),
}

impl Field {
    pub fn new(ty: impl Into<Ident>, id: impl Into<Ident>) -> Self {
        Self { ty: Type::Basic(ty.into()), id: id.into(), bits: Default::default(), _ne: () }
    }

    pub fn new_agg(ty: impl Into<AggregateData>, id: impl Into<Ident>) -> Self {
        Self { ty: Type::Aggregate(ty.into()), id: id.into(), bits: Default::default(), _ne: () }
    }
}

impl Debug for Field {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "Field {{ ty: {:?}, id: {:?}", self.ty, self.id)?;
        if let Some(bits) = self.bits { write!(fmt, ", bits: {}", bits)?; }
        write!(fmt, ", ... }}")
    }
}
