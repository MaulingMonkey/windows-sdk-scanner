use std::fmt::{self, Debug, Display, Formatter};
use std::path::*;
use std::sync::*;
use std::num::*;



/// `file:line:col`
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    pub path:       Arc<Path>,
    pub line_no:    Option<NonZeroUsize>,
    pub col_no:     Option<NonZeroUsize>,
}

impl Location {
    pub fn line_no_or_0(&self) -> usize { self.line_no.map_or(0, |l| l.get()) }
}

impl Default for Location {
    fn default() -> Self { Self { path: Path::new("").into(), line_no: None, col_no: None } }
}

impl Debug for Location {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "`{}`", self)
    }
}

impl Display for Location {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", self.path.display())?;
        if let Some(line_no) = self.line_no {
            write!(fmt, ":{}", line_no)?;
            if let Some(col_no) = self.col_no {
                write!(fmt, ":{}", col_no)?;
            }
        }
        Ok(())
    }
}
