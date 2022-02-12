use maulingmonkey_windows_sdk_scanner::*;

fn main() {
    for sdk in sdk::WindowsKit::find_all().unwrap().iter() {
        let xaudio2_h = sdk.include.join(r"um\xaudio2.h");
        if !xaudio2_h.exists() { continue }
        let mut root = RootBuilder::new();
        root.add_from_cpp_path(&xaudio2_h).unwrap();
        root.add_from_cpp_path(&sdk.include.join(r"um\unknwnbase.h")).unwrap();
        let _root = root.finish();
    }
}
