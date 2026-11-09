yazi_macro::mod_flat!(
	app cache call hold http image iter json layer log preview process regex shell spot sync target tasks text time user utils
);

pub(crate) fn shutdown() { HELD.lock().clear(); }
