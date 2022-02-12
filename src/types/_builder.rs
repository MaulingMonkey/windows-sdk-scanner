use crate::*;

use std::io;
use std::path::*;



/// Scan C++ code, eventually resulting in a [`Root`].
pub struct RootBuilder(Root);
// We make `Root` private mostly to ensure `.cleanup()` is called before making [`Root`]'s data publicly available.

impl RootBuilder {
    pub fn new() -> Self {
        Self(Root::default())
    }

    pub fn finish(mut self) -> Root {
        self.0.cleanup();
        self.0
    }

    /// Add/scan an individual C++ header.
    #[inline] pub fn add_from_cpp_path(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        self.0.add_from_cpp_path(path.as_ref())
    }

    /// Add/scan an entire C++ directory.
    #[inline] pub fn add_from_dir(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        for e in std::fs::read_dir(path)? {
            let e = e?;
            let file_type = e.file_type()?;
            if file_type.is_dir() {
                self.add_from_dir(e.path())?;
            }
            if file_type.is_file() {
                let name = e.file_name();
                let name_lossy = name.to_string_lossy();
                if ".h .hpp".split(' ').any(|ext| name_lossy.ends_with(ext)) {
                    self.add_from_cpp_path(e.path())?;
                }
            }
        }
        Ok(())
    }

    /// Add/scan a Windows SDK.
    /// Eventually this should scan the entire SDK, but for now may scan a known working subset.
    pub fn add_from_sdk(&mut self, sdk: &sdk::WindowsKit, force_all: bool) -> io::Result<()> {
        if force_all { return self.add_from_dir(&sdk.include); }

        for header in [
            // misc
            r"shared\guiddef.h",
            r"um\unknwnbase.h",
            r"um\winuser.h",

            // d3d
            r"um\d3dcaps.h",
            r"um\d3dcommon.h",
            r"um\d3dcsx.h",
            r"um\d3dhal.h",
            r"um\d3dhalex.h",
            r"um\d3dnthal.h",
            r"um\d3dtypes.h",

            // d3d9
            r"shared\d3d9.h",
            r"shared\d3d9caps.h",
            r"shared\d3d9types.h",

            // d3d11
            r"um\d3d11.h",
            r"um\d3d11shader.h",
            r"um\d3d11shadertracing.h",

            // d3d12
            r"um\d3d12.h",
            r"um\d3d12sdklayers.h",
            r"um\d3d12shader.h",
            r"um\d3d12video.h",

            // d3dcompiler
            r"um\d3dcompiler.h",

            // dinput
            r"um\dinput.h",

            // dxcompiler
            // ???

            // dxgi
            r"shared\dxgi.h",
            r"shared\dxgi1_2.h",
            r"shared\dxgi1_3.h",
            r"shared\dxgi1_4.h",
            r"shared\dxgi1_5.h",
            r"shared\dxgi1_6.h",
            r"shared\dxgiformat.h",
            r"shared\dxgitype.h",

            // xaudio2
            r"um\xaudio2.h",
            r"um\xaudio2fx.h",

            // xinput
            r"um\xinput.h",
        ].into_iter() {
            let path = sdk.include.join(header);
            if path.exists() { self.add_from_cpp_path(path)?; }
        }
        Ok(())
    }
}
