yazi_macro::mod_pub!(cha error mounts path provider);

yazi_macro::mod_flat!(cwd file files filter fns hash op scheme sorter sorting splatter stage url xdg);

pub fn init() {
	CWD.init(<_>::default());

	mounts::init();
}
