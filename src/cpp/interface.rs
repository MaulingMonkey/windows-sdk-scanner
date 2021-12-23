use crate::*;

use mmrbi::*;

use std::fmt::{self, Debug, Formatter};
use std::io;



/// `interface id : public base { ... }` or ...
pub struct Interface {
    /// A class/interface name such as `IUnknown`.
    pub id:                     Ident,

    /// The base class/interface, if any, such as `IUnknown`.
    pub base:                   Option<Ident>,

    /// The methods belonging to this interface.
    /// May include methods inherited from `base` if not cleaned up.
    pub(crate) all_methods:     VecMap<Ident, Method>,

    pub(crate) _non_exhaustive: (),
}

impl Debug for Interface {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("Interface")
            .field("id",            &self.id                        )
            .field("base",          &self.base                      )
            .field("methods",       &self.methods().map(|m| m.f.id.as_str()).collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}



impl Interface {
    pub fn valid_name(name: &str) -> bool { valid_name(name) && !name.ends_with("Vtbl") }

    pub fn new(id: Ident, base: Option<Ident>) -> Self {
        Self {
            id, base,
            all_methods: Default::default(),
            _non_exhaustive: ()
        }
    }

    pub fn methods(&self) -> impl Iterator<Item = &Method> { self.all_methods.values_by_key().filter(|m| !m.is_inherited()) }

    pub(crate) fn add_from_cpp(&mut self, interface_start: &Location, src: &mut SrcReader) -> io::Result<()> {
        while let Some(SrcLine { location, trimmed, .. }) = src.next_line() {
            if trimmed == "" { continue }
            if trimmed == "{" { break }
            return Err(err_expected(&location, "`{` after `*_INTERFACE(...)` macro"));
        }

        while let Some(SrcLine { location, trimmed, .. }) = src.next_line() {
            if trimmed == "}" { return Ok(()) }
            if trimmed == "};" { return Ok(()) }

            // e.g.:        `virtual HRESULT STDMETHODCALLTYPE GetFormat( `
            // but exclude: `HRESULT ( STDMETHODCALLTYPE *GetFormat )(`
            if let Some(method) = trimmed.split_once_trim(" STDMETHODCALLTYPE ")   // find abi marker
                .filter(|(_, after)| !after.starts_with("*"))                   // exclude function pointers
                .and_then(|(_, after)| after.split_once_trim("("))              // start of arguments list
                .map(|(before, _params)| before.try_rsplit_once_trim(" "))      // find start of method name
                .map(|(_prev, method)| method)
            {
                if Method::valid_name(method) {
                    let method = Method::new(self.id.clone(), Ident::own(method));
                    self.add_method(&location, method);
                }
                continue
            }

            // STDMETHOD(QueryInterface)(
            if let Some(method) = trimmed.split_once_trim("STDMETHOD(")
                .and_then(|(_, after)| after.split_once_trim(")"))
                .map(|(method, _)| method)
            {
                if Method::valid_name(method) {
                    let method = Method::new(self.id.clone(), Ident::own(method));
                    self.add_method(&location, method);
                }
                continue
            }

            // STDMETHOD_(ULONG, AddRef)(
            if let Some((_ret, method)) = trimmed.split_once_trim("STDMETHOD_(")
                .and_then(|(_, after)| after.split_once_trim(")"))
                .and_then(|(ret_method, _)| ret_method.split_once_trim(","))
            {
                if Method::valid_name(method) {
                    let method = Method::new(self.id.clone(), Ident::own(method));
                    self.add_method(&location, method);
                }
                continue
            }
        }

        Err(unexpected_eof(&interface_start, &format!("closing `}}` for interface `{}`", self.id)))
    }

    fn add_method(&mut self, location: &Location, method: Method) {
        debug_assert_eq!(method.ty, self.id);
        if let Some(prev) = self.all_methods.insert(method.f.id.clone(), method) {
            warning!(at: &location.path, line: location.line_no_or_0(), "duplicate method `{}::{}`", prev.ty, prev.f.id);
        }
    }
}
