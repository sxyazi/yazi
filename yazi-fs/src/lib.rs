extern crate self as yazi_fs;

yazi_macro::mod_pub!(stat casefold copier remover file files mounts path engine scanner trash op splatter);

yazi_macro::mod_flat!(auth cwd entries filter fns hash normalizer sorter sorting stage url xdg);

pub fn init() {
	CWD.init(<_>::default());

	Xdg::load();
}
