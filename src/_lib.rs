// ...

macro_rules! mods {
    ( $( #[$attr:meta] )* inl      mod $mod:ident ;                $($tt:tt)* ) => { $(#[$attr])*      mod $mod;                       #[allow(unused_imports)] pub use $mod::*; mods!{ $($tt)* } };
    ( $( #[$attr:meta] )* inl      mod $mod:ident { $($body:tt)* } $($tt:tt)* ) => { $(#[$attr])*      mod $mod { mods!{ $($body)* } } #[allow(unused_imports)] pub use $mod::*; mods!{ $($tt)* } };
    ( $( #[$attr:meta] )* $vis:vis mod $mod:ident ;                $($tt:tt)* ) => { $(#[$attr])* $vis mod $mod;                                                                 mods!{ $($tt)* } };
    ( $( #[$attr:meta] )* $vis:vis mod $mod:ident { $($body:tt)* } $($tt:tt)* ) => { $(#[$attr])* $vis mod $mod { mods!{ $($body)* } }                                           mods!{ $($tt)* } };
    () => {};
}

use cpp::*;

mods! {
    /// Representations of C++ source code
    pub mod cpp {
        inl mod constant;
        inl mod enum_;
        inl mod field;
        inl mod flag;
        inl mod function;
        inl mod ident;
        inl mod interface;
        inl mod issue;
        inl mod macro_;
        inl mod method;
        inl mod namespace;
        inl mod struct_;
        inl mod type_;
        inl mod union_;
    }

    inl mod ext {
        inl mod char_ext;
        inl mod str_ext;
    }

    inl mod types {
        inl mod _builder;
        inl mod _root;
    }

    inl mod errors;
    inl mod location;
    pub mod sdk;
    inl mod src_reader;
    inl mod validation;
    inl mod vec_map_;
    inl mod version;
        mod ztest;
}
