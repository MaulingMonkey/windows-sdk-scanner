use crate::*;

use mmrbi::*;

use std::fmt::{self, Debug, Formatter};
use std::io;
use std::path::*;
use std::sync::*;



/// { interfaces, structs, flags, enums, macros, constants, ... }
#[derive(Default)]
pub struct Root {
    pub interfaces:             VecMap<Ident, Interface>,
    pub structs:                VecMap<Ident, Struct>,
    pub unions:                 VecMap<Ident, Union>,
    pub flags:                  VecMap<Ident, Flags>,
    pub enums:                  VecMap<Ident, Enum>,
    pub macros:                 VecMap<Ident, Macro>,
    pub constants:              VecMap<Ident, Constant>,
    pub namespaces:             VecMap<Ident, Namespace>,
    pub functions:              VecMap<Ident, Function>,
    pub(crate) _non_exhaustive: (),
}

impl Debug for Root {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("Root")
            .field("interfaces",    &self.interfaces    .values_by_key().collect::<Vec<_>>())
            .field("structs",       &self.structs       .values_by_key().collect::<Vec<_>>())
            .field("unions",        &self.unions        .values_by_key().collect::<Vec<_>>())
            .field("flags",         &self.flags         .values_by_key().collect::<Vec<_>>())
            .field("macros",        &self.macros        .values_by_key().collect::<Vec<_>>())
            .field("enums",         &self.enums         .values_by_key().collect::<Vec<_>>())
            .field("constants",     &self.constants     .values_by_key().collect::<Vec<_>>())
            .field("namespaces",    &self.namespaces    .values_by_key().collect::<Vec<_>>())
            .field("functions",     &self.functions     .values_by_key().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}



impl Root {
    pub fn new() -> Self { Self::default() }

    /// Mark inherited methods etc.
    pub(crate) fn cleanup(&mut self) {
        self.cleanup_inherited_methods()
    }

    #[inline] pub(crate) fn add_from_cpp_path(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        self.impl_add_from_cpp_path(path.as_ref())
    }

    fn cleanup_inherited_methods(&mut self) {
        for interface in self.interfaces.values_by_key() {
            let mut next_base = &interface.base;
            while let Some(base) = next_base.as_ref().and_then(|base| self.interfaces.get(base)) {
                for method in base.all_methods.keys().filter_map(|m| interface.all_methods.get(m)) {
                    method.inherited.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                next_base = &base.base;
            }
        }
    }

    fn impl_add_from_cpp_path(&mut self, path: &Path) -> io::Result<()> {
        let path = Arc::from(path);
        let all = std::fs::read_to_string(&path)?;
        let mut src = SrcReader::new(path.clone(), &all);

        'file: while let Some(line) = src.next_line() {
            let unexpected_eof  = |e| unexpected_eof(&line.location, e);
            let warn_expected   = |e| warn_expected(&line.location, e);

            if let Some(_pp) = line.trimmed.strip_prefix("#") {
                // Preprocessor command (#ifdef, #if, #else, #endif, #define, #include, etc.)
            } else if let Some(_cpp_comment) = line.trimmed.strip_prefix("//") {
                // C++ style single line comment
            } else if let Some(interface) = line.trimmed.strip_prefix_suffix("DECLARE_INTERFACE(", ")") {
                let mut interface = Interface::new(Ident::own(interface), None);
                let err = interface.add_from_cpp(&line.location, &mut src);
                self.add_interface(&line.location, interface);
                err?;
            } else if let Some(interface_base) = line.trimmed.strip_prefix_suffix("DECLARE_INTERFACE_(", ")") {
                let (interface, base) = interface_base.split_once_trim(",").ok_or_else(|| warn_expected("comma in `DECLARE_INTERFACE_(Interface, Base)`"))?;
                let mut interface = Interface::new(Ident::own(interface), Some(Ident::own(base)));
                let err = interface.add_from_cpp(&line.location, &mut src);
                self.add_interface(&line.location, interface);
                err?;
            } else if let Some(_midl) = line.trimmed.strip_prefix_suffix("MIDL_INTERFACE(\"", "\")") {
                let line = src.next_line().ok_or_else(|| unexpected_eof("interface line following `MIDL_INTERFACE(\"...\")`"))?;
                if let Some((interface, vis_base)) = line.trimmed.split_once_trim(":") {
                    let mut interface = Interface::new(Ident::own(interface), None);
                    if let Some((vis, base)) = vis_base.split_once_trim(" ") {
                        match vis {
                            "public"    => {},
                            "protected" => drop(warn_expected("public, not protected")),
                            "private"   => drop(warn_expected("public, not private")),
                            _other      => drop(warn_expected("public, protected, or private inheritance")),
                        }
                        interface.base = Some(Ident::own(base));
                    } else {
                        interface.base = Some(Ident::own(vis_base));
                    }
                    let err = interface.add_from_cpp(&line.location, &mut src);
                    self.add_interface(&line.location, interface);
                    err?;
                } else if line.trimmed == "IUnknown" {
                    let mut interface = Interface::new(Ident::own(line.trimmed), None);
                    let err = interface.add_from_cpp(&line.location, &mut src);
                    self.add_interface(&line.location, interface);
                    err?;
                } else {
                    warn_expected("`Interface : Base` line following `MIDL_INTERFACE(\"...\")`");
                }
            } else if let Some(winapi) = line.trimmed.find_token("WINAPI") {
                //let _ret = line.trimmed[..winapi].trim_end();
                let rest = line.trimmed[winapi+6..].trim_start();
                let func_end = rest.find('(').unwrap_or(rest.len());
                let func = rest[..func_end].trim_end();
                if !Function::valid_name(func) { continue 'file }
                let mut func = Function::new(Ident::own(func));
                func.abi = FunctionAbi::Winapi;
                self.add_function(&line.location, func);
            } else if let Some(name) = line.trimmed.strip_prefix_suffix("typedef struct ", "{").map(str::trim) {
                // FIXME: `enum` might be on next line (e.g. `D3D11_AUTHENTICATED_PROCESS_IDENTIFIER_TYPE`)
                // FIXME: `{` might be on next line (e.g. `D3D11_AUTHENTICATED_PROTECTION_FLAGS`)
                let mut s = Struct::new(Ident::own(name));
                let err = s.add_from_cpp(&line.location, &mut src, true);
                self.add_structure(&line.location, s);
                err?;
            } else if let Some(name) = line.trimmed.strip_prefix_suffix("typedef union ", "{").map(str::trim) {
                let mut u = Union::new(Ident::own(name));
                let err = u.add_from_cpp(&line.location, &mut src, true);
                self.add_union(&line.location, u);
                err?;
            } else if let Some(name) = line.trimmed.strip_prefix_suffix("typedef enum ", "{").map(str::trim) {
                let mut e = Enum::new(Ident::own(name));
                let err = e.add_from_cpp(&line.location, &mut src, true);
                self.add_enumeration(&line.location, e);
                err?;
            } else {
                // flags? ...?
            }
        }

        Ok(())
    }

    fn add_interface(&mut self, loc: &Location, mut interface: Interface) {
        let path = &*loc.path;
        let line_no = loc.line_no_or_0();

        match self.interfaces.entry(interface.id.clone()) {
            vec_map::Entry::Vacant(entry) => {
                interface.defined_at.insert(loc.clone());
                entry.insert(interface);
            },
            vec_map::Entry::Occupied(mut entry) => {
                let prev = entry.get_mut();
                prev.defined_at.insert(loc.clone());
                let mut new_methods  = interface.methods().map(|m| m.f.id.as_str());
                let mut prev_methods = prev     .methods().map(|m| m.f.id.as_str());
                let interface = &interface.id;
                loop {
                    match (prev_methods.next(), new_methods.next()) {
                        (Some(prev), Some(new)) if prev < new => warning!(at: path, line: line_no, "duplicate interface `{}` missing previous method `{}`", interface, prev),
                        (Some(prev), Some(new)) if prev > new => warning!(at: path, line: line_no, "duplicate interface `{}` adds new method `{}`", interface, new),
                        (Some(_prev), Some(_new)) => continue,

                        (Some(prev), None)  => warning!(at: path, line: line_no, "duplicate interface `{}` missing previous method `{}`", interface, prev),
                        (None, Some(new))   => warning!(at: path, line: line_no, "duplicate interface `{}` adds new method `{}`", interface, new),
                        (None, None)        => {},
                    }
                    break;
                }
            },
        }
    }

    fn add_function(&mut self, loc: &Location, mut function: Function) {
        match self.functions.entry(function.id.clone()) {
            vec_map::Entry::Vacant(entry) => {
                function.defined_at.insert(loc.clone());
                entry.insert(function);
            },
            vec_map::Entry::Occupied(mut entry) => {
                let prev = entry.get_mut();
                if function.abi != prev.abi { warning!(at: &loc.path, line: loc.line_no_or_0(), "duplicate function declaration for `{}` has varying ABI: {:?} vs {:?}", function.id, prev.abi, function.abi) }
                // TODO: ret, params?
                prev.defined_at.insert(loc.clone());
            },
        }
    }

    fn add_structure(&mut self, _loc: &Location, structure: Struct) {
        match self.structs.entry(structure.id.clone()) {
            vec_map::Entry::Vacant(entry) => drop(entry.insert(structure)),
            vec_map::Entry::Occupied(entry) => {
                let _prev = entry.get();
                // TODO: fields, layout?
            },
        }
    }

    fn add_union(&mut self, _loc: &Location, u: Union) {
        match self.unions.entry(u.id.clone()) {
            vec_map::Entry::Vacant(entry) => drop(entry.insert(u)),
            vec_map::Entry::Occupied(entry) => {
                let _prev = entry.get();
                // TODO: fields, layout?
            },
        }
    }

    fn add_enumeration(&mut self, _loc: &Location, e: Enum) {
        match self.enums.entry(e.id.clone()) {
            vec_map::Entry::Vacant(entry) => drop(entry.insert(e)),
            vec_map::Entry::Occupied(entry) => {
                let _prev = entry.get();
                // TODO: values, abi?
            },
        }
    }
}
