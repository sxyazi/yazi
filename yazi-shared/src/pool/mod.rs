yazi_macro::mod_flat!(cow pool ptr symbol traits);

const HASHER: foldhash::fast::FixedState = foldhash::fast::FixedState::with_seed(0);

static SYMBOLS: parking_lot::Mutex<hashbrown::HashMap<SymbolPtr, u64, foldhash::fast::FixedState>> =
	parking_lot::Mutex::new(hashbrown::HashMap::with_hasher(HASHER));

#[inline]
pub(super) fn compute_hash<T: std::hash::Hash>(value: T) -> u64 {
	use core::hash::BuildHasher;
	HASHER.hash_one(value)
}
