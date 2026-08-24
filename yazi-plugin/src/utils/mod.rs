yazi_macro::mod_flat!(
	app cache call history hold http image json layer log preview process spot sync target tasks text time user utils
);

pub(crate) fn shutdown() { HELD.lock().clear(); }
