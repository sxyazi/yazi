use arc_swap::{ArcSwap, ArcSwapAny, Guard, RefCnt, strategy::CaS};

// --- IntoPointee
pub trait IntoPointee: Sized {
	fn into_pointee(self) -> ArcSwap<Self>;
}

impl<T> IntoPointee for T {
	#[inline]
	fn into_pointee(self) -> ArcSwap<Self> { ArcSwap::from_pointee(self) }
}

// --- ArcSwapExt
pub trait ArcSwapExt<T: RefCnt> {
	fn try_rcu<R, E, F>(&self, f: F) -> Result<T, E>
	where
		F: FnMut(&T) -> Result<R, E>,
		R: Into<T>;
}

impl<T, S> ArcSwapExt<T> for ArcSwapAny<T, S>
where
	T: RefCnt,
	S: CaS<T>,
{
	fn try_rcu<R, E, F>(&self, mut f: F) -> Result<T, E>
	where
		F: FnMut(&T) -> Result<R, E>,
		R: Into<T>,
	{
		let mut cur = self.load();
		loop {
			let new = f(&cur)?.into();
			let prev = self.compare_and_swap(&*cur, new);
			if T::as_ptr(&*cur) == T::as_ptr(&*prev) {
				return Ok(Guard::into_inner(prev));
			}

			cur = prev;
		}
	}
}
