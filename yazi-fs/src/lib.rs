extern crate self as yazi_fs;

yazi_macro::mod_pub!(cha file mounts path engine);

yazi_macro::mod_flat!(cwd spec entries filter fns hash op sorter sorting splatter stage url xdg);

pub fn init() {
	CWD.init(<_>::default());

	mounts::init();

	Xdg::load();
}
