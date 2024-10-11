#[macro_export]
macro_rules! mod_use {
    [ $( $name:ident $(,)? )+ ] => {
        $(
            mod $name;
            pub use $name::*;
        )+
    };
}
