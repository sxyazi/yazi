yazi_macro::mod_flat!(linked local);

pub static LINKED: parking_lot::RwLock<Linked> = parking_lot::RwLock::new(Linked::new());
