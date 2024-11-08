use mlua::{AnyUserData, UserData};

use super::Lives;

pub(super) struct Iter<I: Iterator<Item = T>, T> {
	inner: I,
	count: usize,
}

impl<I: Iterator<Item = T> + 'static, T: 'static> Iter<I, T> {
	#[inline]
	pub(super) fn make(inner: I) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner, count: 0 })
	}
}

impl<I: Iterator<Item = T>, T> Iterator for Iter<I, T> {
	type Item = (usize, T);

	fn next(&mut self) -> Option<Self::Item> {
		let next = self.inner.next()?;
		self.count += 1;
		Some((self.count, next))
	}
}

impl<I: Iterator<Item = T>, T> UserData for Iter<I, T> {}
