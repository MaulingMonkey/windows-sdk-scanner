use std::cmp::Ordering;
use std::fmt::{self, Debug, Display, Formatter};
use std::path::*;



/// A validated, properly sorting version like "10.0.18362.0"
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Version(String);

impl Version {
    pub fn parse(s: impl Into<String>) -> Result<Self, ()> {
        let s = s.into();
        if s.is_empty() {
            Err(())
        } else if s.bytes().all(|b| b == b'.' || (b'0' <= b && b <= b'9')) {
            Ok(Self(s))
        } else {
            Err(())
        }
    }

    pub fn as_str(&self) -> &str { &*self.0 }
    pub fn as_path(&self) -> &Path { Path::new(&self.0) }
}

impl AsRef<Path> for Version {
    fn as_ref(&self) -> &Path { self.as_path() }
}

impl Debug   for Version { fn fmt(&self, fmt: &mut Formatter) -> fmt::Result { Debug  ::fmt(self.as_str(), fmt) } }
impl Display for Version { fn fmt(&self, fmt: &mut Formatter) -> fmt::Result { Display::fmt(self.as_str(), fmt) } }

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut a = self.0.split('.');
        let mut b = other.0.split('.');
        loop {
            match (a.next(), b.next()) {
                (Some(a), Some(b)) if a == b    => continue,
                (Some(a), Some(b))              => return a.parse::<u32>().unwrap_or(0).cmp(&b.parse::<u32>().unwrap_or(0)),
                (Some(_), None)                 => return Ordering::Greater,    // "1.0._" > "1.0"
                (None, Some(_))                 => return Ordering::Less,       // "1.0" < "1.0._"
                (None, None)                    => return Ordering::Equal,      // "1.0" == "1.0"
            }
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}



/// A [`Version`] failed to parse.
#[derive(Clone, Debug)]
pub struct ParseVersionError(());

impl Display for ParseVersionError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "ParseVersionError")
    }
}

impl std::error::Error for ParseVersionError {}



#[test] fn test_cmp() {
    let a = Version::parse("10.0").unwrap();
    let b = Version::parse("10.0.18362.0").unwrap();
    let c = Version::parse("10.0.19041.0").unwrap();

    assert!(a < b);
    assert!(a < c);
    assert!(b < c);

    assert!(a == a);
    assert!(b == b);
    assert!(c == c);

    assert!(b > a);
    assert!(c > a);
    assert!(c > b);
}
