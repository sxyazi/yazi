#![allow(clippy::if_same_then_else, clippy::option_map_unit_fn)]

extern crate self as yazi_actor;

yazi_macro::mod_pub!(cmp confirm core help input lives mgr pick spot tasks which);

yazi_macro::mod_flat!(actor context);
