use crate::{scheme::Encode, url::Url};

pub struct Display<'a> {
	inner: Url<'a>,
}

impl<'a> From<Url<'a>> for Display<'a> {
	fn from(value: Url<'a>) -> Self { Self { inner: value } }
}

impl<'a> std::fmt::Display for Display<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let (kind, loc) = (self.inner.kind(), self.inner.loc());
		if kind.is_virtual() {
			Encode(self.inner).fmt(f)?;
		}
		loc.display().fmt(f)
	}
}
