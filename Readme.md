# `maulingmonkey-windows-sdk-scanner`

[![GitHub](https://img.shields.io/github/stars/MaulingMonkey/windows-sdk-scanner.svg?label=GitHub&style=social)](https://github.com/MaulingMonkey/windows-sdk-scanner)
[![Build Status](https://github.com/MaulingMonkey/windows-sdk-scanner/workflows/Rust/badge.svg)](https://github.com/MaulingMonkey/windows-sdk-scanner/actions?query=workflow%3Arust)
<!--
[![License](https://img.shields.io/crates/l/maulingmonkey-windows-sdk-scanner.svg)](https://github.com/MaulingMonkey/windows-sdk-scanner)
[![crates.io](https://img.shields.io/crates/v/maulingmonkey-windows-sdk-scanner.svg)](https://crates.io/crates/maulingmonkey-windows-sdk-scanner)
[![docs.rs](https://docs.rs/maulingmonkey-windows-sdk-scanner/badge.svg)](https://docs.rs/maulingmonkey-windows-sdk-scanner)
[![dependency status](https://deps.rs/repo/github/MaulingMonkey/windows-sdk-scanner/status.svg)](https://deps.rs/repo/github/MaulingMonkey/windows-sdk-scanner)
-->

Unstable APIs for scaning the Windows SDK



### Quickstart

```toml
# Cargo.toml
[dependencies]
maulingmonkey-windows-sdk-scanner.git = "https://github.com/MaulingMonkey/windows-sdk-scanner"
```

```rust
// *.rs
use maulingmonkey_windows_sdk_scanner::*;
let sdk = sdk::WindowsKit::find_latest().unwrap();
let mut cpp = RootBuilder::new();
cpp.add_from_sdk(&sdk, false).unwrap();
let cpp : Root = cpp.finish();
for s in cpp.structs.values_by_key() { dbg!(s); }
```

### Why?

*   Windows SDK headers are the closest thing to being the "truth on the ground."
*   I want to cross-check / verify my Rust api generation against C++ stuff.
*   Autogen / autoverify `winapi` to relieve stress on the bunny?
*   `*.winmd` stuff is cool but complicated as heck to use



### Alternatives

| Crate | Desc |
| ----- | ---- |
| [`winapi`](https://lib.rs/crates/winapi)                                  | hand-authored bindings
| [`windows`](https://lib.rs/crates/windows)                                | semi-generated bindings
| [`winmd`](https://lib.rs/crates/winmd)                                    | (abandoned?) \*.winmd parser
| [`windows_reader`](https://docs.rs/windows_reader/latest/windows_reader/) | \*.winmd parser (poorly documented)



### Progress

|  ?  | Category    | Notes |
| --- | ----------- | ----- |
| ⚠️ | interfaces   |
| ⚠️ | methods      | No signature information, `STDCALL[_]` marked only
| ⚠️ | structs      | Must have the form `typedef struct ... { ... } ...;`
| ⚠️ | functions    | `WINAPI` marked only
| ⚠️ | unions       | Must have the form `typedef union ... { ... } ...;`
| ⚠️ | enums        | Must have the form `typedef enum ... { ... } ...;`
| ❌ | flags        |
| ❌ | constants    |
| ❌ | macros       |
| ❌ | namespaces   |

⚠️ Caveats ⚠️
*   Bitfields are not (yet?) supported
*   Anonymous interior structs/unions are not (yet?) supported
*   Preprocessor `#if ... #endif` junk within types is not (yet?) supported



<h2 name="license">License</h2>

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.



<h2 name="contribution">Contribution</h2>

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
