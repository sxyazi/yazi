#![allow(clippy::if_same_then_else, clippy::option_map_unit_fn)]

yazi_macro::mod_pub!(cha mounts provider path);

yazi_macro::mod_flat!(calculator cwd file files filter fns op sorter sorting stage xdg);

pub fn init() {
	CWD.init(<_>::default());

	mounts::init();
}
