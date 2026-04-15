use arc_swap::ArcSwap;

// --- IntoPointee
pub trait IntoPointee: Sized {
	fn into_pointee(self) -> ArcSwap<Self>;
}

impl<T> IntoPointee for T {
	#[inline]
	fn into_pointee(self) -> ArcSwap<Self> { ArcSwap::from_pointee(self) }
}
