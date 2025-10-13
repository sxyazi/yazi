yazi_macro::mod_flat!(linked local);

pub static LINKED: yazi_shared::RoCell<parking_lot::RwLock<Linked>> = yazi_shared::RoCell::new();

pub(super) fn init() { LINKED.with(<_>::default); }
