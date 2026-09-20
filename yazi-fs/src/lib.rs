extern crate self as yazi_fs;

yazi_macro::mod_pub!(stat casefold file mounts path engine scanner trash op);

yazi_macro::mod_flat!(auth cwd entries filter fns hash normalizer sorter sorting splatter stage url xdg);

pub fn init() {
	CWD.init(<_>::default());

	mounts::init();

	Xdg::load();
}
