use crate::*;

use std::fmt::{self, Debug, Formatter};



pub struct Issue {
    pub location:   Location,
    pub message:    String,
    // TODO: severity?
    _ne:            (),
}

impl Issue {
    pub fn new(location: Location, message: impl Into<String>) -> Self {
        let message = message.into();
        Self { location, message, _ne: () }
    }
}

impl Debug for Issue {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "Issue {{ location: `{}`, message: {:?}, .. }}", self.location, self.message)
    }
}
