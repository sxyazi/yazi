#![allow(clippy::if_same_then_else, clippy::option_map_unit_fn, clippy::unit_arg)]

yazi_macro::mod_pub!(cha mounts provider path);

yazi_macro::mod_flat!(cwd file files filter fns op sorter sorting stage);

pub fn init() {
	CWD.init(<_>::default());

	mounts::init();

	provider::init();
}
