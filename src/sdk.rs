//! [`WindowsKit`]

use crate::*;

use std::io;
use std::path::*;



pub struct WindowsKit {
    /// e.g. `10.0.19041.0`
    pub sdk_version: Version,

    /// e.g. `C:\Program Files (x86)\Windows Kits\10\Include\10.0.19041.0\`
    pub include: PathBuf,
}

impl WindowsKit {
    /// Find e.g. `C:\Program Files (x86)\Windows Kits\10\Include\10.0.19041.0\`
    pub fn find_all() -> io::Result<Vec<Self>> {
        let mut paths = Vec::new();
        for env in [
            "ProgramFiles(x86)",
            "ProgramFiles",
        ].into_iter() {
            let mut include = if let Some(v) = std::env::var_os(env) { PathBuf::from(v) } else { continue };
            include.push(r"Windows Kits\10\Include");

            match std::fs::read_dir(&include) {
                Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
                Err(err) => return Err(err),
                Ok(dir) => {
                    for e in dir {
                        let e = e?;
                        if !e.file_type()?.is_dir() { continue }
                        let sdk_version = if let Ok(v) = Version::parse(e.file_name().into_string().unwrap_or(String::new())) { v } else { continue };
                        paths.push(Self {
                            include: include.join(sdk_version.as_path()),
                            sdk_version,
                        });
                    }
                },
            }

            // if !paths.is_empty() { break }
        }
        paths.sort_unstable_by(|a, b| a.sdk_version.cmp(&b.sdk_version));
        Ok(paths)
    }

    /// e.g. `C:\Program Files (x86)\Windows Kits\10\Include\10.0.19041.0\{cppwinrt, shared, ucrt, um, winrt}\`
    pub fn includes<'s>(&'s self) -> impl Iterator<Item = PathBuf> + 's {
        "cppwinrt shared ucrt um winrt".split(' ').flat_map(|sub| {
            let p = self.include.join(sub);
            p.exists().then(move || p)
        })
    }

    pub fn headers(&self) -> io::Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        for include in self.includes() {
            for e in std::fs::read_dir(&include)? {
                let e = e?;
                let file_type = e.file_type()?;
                if file_type.is_dir() {
                    // TODO: enumerate subdirs
                }
                if file_type.is_file() {
                    let file_name = e.file_name();
                    let file_name_lossy = file_name.to_string_lossy();
                    if ".h .hpp .inl .idl".split(' ').any(|ext| file_name_lossy.ends_with(ext)) {
                        // C++ headers, inline files, IDL files
                        paths.push(e.path());
                    }
                }
            }
        }
        Ok(paths)
    }
}



#[test] fn test_find() {
    let sdks = WindowsKit::find_all().unwrap();
    assert!(!sdks.is_empty());

    for sdk in sdks.iter() {
        assert!(sdk.includes().any(|_| true), "SDK has no include directories\r\n    {}\\\r\n", sdk.include.display());
    }
}
