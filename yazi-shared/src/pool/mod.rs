yazi_macro::mod_flat!(pool ptr symbol traits);

static SYMBOLS: crate::RoCell<parking_lot::Mutex<hashbrown::HashMap<SymbolPtr, u64>>> =
	crate::RoCell::new();

pub(super) fn init() { SYMBOLS.with(<_>::default); }
