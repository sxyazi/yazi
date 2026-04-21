yazi_macro::mod_flat!(linked local);

pub static LINKED: yazi_shim::cell::RoCell<parking_lot::RwLock<Linked>> =
	yazi_shim::cell::RoCell::new();

pub(super) fn init() { LINKED.with(<_>::default); }
