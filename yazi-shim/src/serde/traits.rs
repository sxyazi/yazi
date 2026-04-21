use arc_swap::ArcSwap;
use hashbrown::HashMap;

use crate::cell::SyncCell;

pub trait Overlay<Rhs = Self> {
	fn overlay(&self, new: Rhs);
}

impl<T> Overlay for ArcSwap<T> {
	fn overlay(&self, new: Self) { self.store(new.into_inner()); }
}

impl<T> Overlay for SyncCell<T>
where
	T: Copy,
{
	fn overlay(&self, new: Self) { self.set(new.get()); }
}

impl<T> Overlay for Vec<T> {
	fn overlay(&self, _: Self) {}
}

impl<K, V, S> Overlay for HashMap<K, V, S> {
	fn overlay(&self, _: Self) {}
}
