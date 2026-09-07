use std::{fmt, ops::Deref};

use crate::{spec::Display as DisplaySpec, url::{AsUrl, Url, UrlBuf, UrlCow}};

impl fmt::Display for Url<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Display(*self).fmt(f) }
}

impl fmt::Display for UrlBuf {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.as_url().fmt(f) }
}

impl fmt::Display for UrlCow<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.as_url().fmt(f) }
}

// --- Display
pub struct Display<'a>(pub(crate) Url<'a>);

impl<'a> Deref for Display<'a> {
	type Target = Url<'a>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl fmt::Display for Display<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if !self.is_regular() {
			DisplaySpec(self.0).fmt(f)?;
		}

		self.loc().display().fmt(f)
	}
}
