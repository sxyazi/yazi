extern crate self as yazi_fs;

yazi_macro::mod_pub!(cha file mounts path provider);

yazi_macro::mod_flat!(cwd entries filter fns hash op scheme sorter sorting splatter stage url xdg);

pub fn init() {
	CWD.init(<_>::default());

	mounts::init();

	Xdg::load();
}
