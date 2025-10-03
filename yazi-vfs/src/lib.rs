#![allow(clippy::if_same_then_else, clippy::unit_arg)]

yazi_macro::mod_pub!(provider);

yazi_macro::mod_flat!(cha file files fns op);

pub fn init() { provider::init(); }
