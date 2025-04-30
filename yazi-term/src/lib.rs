#![allow(clippy::unit_arg)]

yazi_macro::mod_pub!(tty);

yazi_macro::mod_flat!(cursor if_);

pub fn init() { tty::init(); }
