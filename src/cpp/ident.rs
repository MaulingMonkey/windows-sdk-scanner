use std::borrow::Borrow;
use std::cmp::*;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::*;
use std::ops::Deref;
use std::sync::Arc;



/// A C++ identifier
#[derive(Clone)] pub struct Ident(IdentImpl);
#[derive(Clone)] enum IdentImpl { Ref(&'static str), Own(Arc<str>) }

impl Ident {
    pub const fn empty() -> Self { Self(IdentImpl::Ref("")) }
    pub fn own(s: &str) -> Self { Self(IdentImpl::Own(s.into())) }
    pub fn from(s: impl Into<Self>) -> Self { s.into() }
    pub fn as_str(&self) -> &str { match &self.0 { IdentImpl::Ref(s) => s, IdentImpl::Own(s) => &*s } }
}

impl From<&'static str> for Ident { fn from(s: &'static str) -> Self { Self(IdentImpl::Ref(s)) } }
impl From<String      > for Ident { fn from(s: String      ) -> Self { Self(IdentImpl::Own(s.into())) } }

impl     From<&   Ident> for String  { fn from(i: &   Ident) -> Self { i.as_str().into() } }
impl<'s> From<&'s Ident> for &'s str { fn from(i: &'s Ident) -> Self { i.as_str() } }

impl AsRef <str> for Ident { fn as_ref(&self) -> &str { self.as_str() } }
impl Borrow<str> for Ident { fn borrow(&self) -> &str { self.as_str() } }
impl Deref for Ident { fn deref(&self) -> &Self::Target { self.as_str() } type Target = str; }

impl Debug      for Ident { fn fmt(&self, fmt: &mut Formatter) -> fmt::Result { Debug::fmt(self.as_str(), fmt) } }
impl Display    for Ident { fn fmt(&self, fmt: &mut Formatter) -> fmt::Result { fmt.write_str(self.as_str()) } }
impl Eq         for Ident {}
impl PartialEq  for Ident { fn eq(&self, other: &Self) -> bool { self.as_str().eq(other.as_str()) } }
impl Ord        for Ident { fn cmp(&self, other: &Self) -> Ordering { self.as_str().cmp(other.as_str()) } }
impl PartialOrd for Ident { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.as_str().partial_cmp(other.as_str()) } }
impl Hash       for Ident { fn hash<H: Hasher>(&self, state: &mut H) { self.as_str().hash(state) } }
