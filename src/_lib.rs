// ...

macro_rules! mods {
    ( inl      mod $mod:ident ; $($tt:tt)* ) => {      mod $mod; #[allow(unused_imports)] pub use $mod::*; mods!( $($tt)* ); };
    ( $vis:vis mod $mod:ident ; $($tt:tt)* ) => { $vis mod $mod; mods!( $($tt)* ); };
    () => {};
}

use cpp::*;
/// Representations of C++ source code
pub mod cpp {
    mods! {
        inl mod constant;
        inl mod enum_;
        inl mod field;
        inl mod flag;
        inl mod function;
        inl mod ident;
        inl mod interface;
        inl mod macro_;
        inl mod method;
        inl mod namespace;
        inl mod struct_;
        inl mod type_;
        inl mod union_;
    }
}

pub(crate) use ext::*;
mod ext {
    mods! {
        inl mod char_ext;
        inl mod str_ext;
    }
}

pub use types::*;
mod types {
    mods! {
        inl mod _builder;
        inl mod _root;
    }
}

mods! {
    inl mod errors;
    inl mod location;
    pub mod sdk;
    inl mod src_reader;
    inl mod validation;
    inl mod vec_map_;
    inl mod version;
        mod ztest;
}
