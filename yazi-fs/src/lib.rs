#![allow(clippy::if_same_then_else, clippy::option_map_unit_fn, clippy::unit_arg)]

yazi_macro::mod_pub!(cha error mounts path provider);

yazi_macro::mod_flat!(cwd file files filter fns op sorter sorting stage url xdg);

pub fn init() {
	CWD.init(<_>::default());

	mounts::init();
}
