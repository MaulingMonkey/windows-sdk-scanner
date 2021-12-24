#![cfg(test)]
#![cfg(windows)]

use crate::*;

use std::fs::File;
use std::io::Write;



#[test] fn test_xaudio2() {
    for sdk in sdk::WindowsKit::find_all().unwrap().iter() {
        let xaudio2_h = sdk.include.join(r"um\xaudio2.h");
        if !xaudio2_h.exists() { continue }
        let mut root = RootBuilder::new();
        root.add_from_cpp_path(&xaudio2_h).unwrap();
        root.add_from_cpp_path(&sdk.include.join(r"um\unknwnbase.h")).unwrap();
        let _root = root.finish();
    }
}

#[test] fn test_more() {
    let _dir = std::fs::create_dir(r"target\testdata");
    for sdk in sdk::WindowsKit::find_all().unwrap().iter() {
        let mut root = RootBuilder::new();
        root.add_from_sdk(&sdk, false).unwrap();
        let root = root.finish();
        let mut o = File::create(format!(r"target\testdata\{}.txt", sdk.sdk_version)).unwrap();
        write!(o, "{:#?}", root).unwrap();
    }
}

#[test] fn test_readme_md_quickstart_compiles() {
    if false {
        //use maulingmonkey_windows_sdk_scanner::*;
        let sdk = sdk::WindowsKit::find_latest().unwrap();
        let mut cpp = RootBuilder::new();
        cpp.add_from_sdk(&sdk, false).unwrap();
        let cpp : Root = cpp.finish();
        for s in cpp.structs.values_by_key() { dbg!(s); }
    }
}
