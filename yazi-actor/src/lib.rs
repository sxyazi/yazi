extern crate self as yazi_actor;

mod macros;

#[doc(hidden)]
pub use {anyhow, paste, yazi_parser};

yazi_macro::mod_pub!(app cmp confirm core help input lives mgr notify pick spot tasks which);

yazi_macro::mod_flat!(actor context);

pub fn init() { lives::init(); }
