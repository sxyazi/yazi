use crate::url::{Url, UrlCow};

pub trait AsUrl {
	fn as_url(&self) -> Url<'_>;
}

impl AsUrl for Url<'_> {
	#[inline]
	fn as_url(&self) -> Url<'_> { *self }
}

impl AsUrl for UrlCow<'_> {
	#[inline]
	fn as_url(&self) -> Url<'_> { self.as_url() }
}

impl AsUrl for &UrlCow<'_> {
	#[inline]
	fn as_url(&self) -> Url<'_> { (*self).as_url() }
}
