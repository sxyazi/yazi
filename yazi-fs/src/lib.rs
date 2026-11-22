extern crate self as yazi_fs;

yazi_macro::mod_pub!(stat casefold copier remover file files mounts path engine identity scanner trash op splatter symlinker);

yazi_macro::mod_flat!(auth cwd entries filter fns hash normalizer sorter sorting stage url xdg);

pub fn init() {
	CWD.init(<_>::default());

	Xdg::load();
}
