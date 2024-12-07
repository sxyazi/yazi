#![allow(clippy::if_same_then_else, clippy::option_map_unit_fn)]

yazi_macro::mod_flat!(cha cwd file files filter fns op path sorter sorting stage step xdg);

pub fn init() { CWD.init(<_>::default()); }
