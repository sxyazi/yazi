use crate::url::Url;

pub struct Display<'a> {
	inner: &'a Url,
}

impl<'a> Display<'a> {
	#[inline]
	pub fn new(inner: &'a Url) -> Self { Self { inner } }
}

impl<'a> std::fmt::Display for Display<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let Url { loc, scheme } = self.inner;
		if scheme.is_virtual() {
			scheme.fmt(f)?;
		}
		loc.display().fmt(f)
	}
}
