yazi_macro::mod_flat!(cow pool ptr symbol traits);

static SYMBOLS: yazi_shim::cell::RoCell<
	parking_lot::Mutex<hashbrown::HashMap<SymbolPtr, u64, foldhash::fast::FixedState>>,
> = yazi_shim::cell::RoCell::new();

pub(super) fn init() { SYMBOLS.with(<_>::default); }

#[inline]
pub(super) fn compute_hash<T: std::hash::Hash>(value: T) -> u64 {
	use core::hash::BuildHasher;
	foldhash::fast::FixedState::default().hash_one(value)
}
