use std::fmt::{self, Debug, Formatter};

use crate::*;



pub enum Type {
    Basic(Ident),
    Aggregate(AggregateData),
    //AnonymousEnum(EnumData),
}

impl Debug for Type {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Type::Basic(id) => Debug::fmt(id, fmt),
            Type::Aggregate(agg) => Debug::fmt(agg, fmt),
        }
    }
}
