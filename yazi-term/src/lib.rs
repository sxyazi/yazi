#![allow(clippy::unit_arg)]

yazi_macro::mod_pub!(tty);

yazi_macro::mod_flat!(cursor r#if);

pub fn init() { tty::init(); }
