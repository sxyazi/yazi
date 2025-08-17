use crate::url::{Encode, UrlBuf};

pub struct Display<'a> {
	inner: &'a UrlBuf,
}

impl<'a> Display<'a> {
	#[inline]
	pub fn new(inner: &'a UrlBuf) -> Self { Self { inner } }
}

impl<'a> std::fmt::Display for Display<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let UrlBuf { loc, scheme } = self.inner;
		if scheme.is_virtual() {
			Encode::from(self.inner).fmt(f)?;
		}
		loc.display().fmt(f)
	}
}
