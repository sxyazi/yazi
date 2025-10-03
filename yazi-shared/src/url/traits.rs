use crate::url::{Url, UrlBuf, UrlCow};

pub trait AsUrl {
	fn as_url(&self) -> Url<'_>;
}

impl AsUrl for Url<'_> {
	#[inline]
	fn as_url(&self) -> Url<'_> { *self }
}

impl AsUrl for UrlBuf {
	#[inline]
	fn as_url(&self) -> Url<'_> { self.as_url() }
}

impl AsUrl for &UrlBuf {
	#[inline]
	fn as_url(&self) -> Url<'_> { (**self).as_url() }
}

impl AsUrl for &mut UrlBuf {
	#[inline]
	fn as_url(&self) -> Url<'_> { (**self).as_url() }
}

impl AsUrl for UrlCow<'_> {
	#[inline]
	fn as_url(&self) -> Url<'_> { self.as_url() }
}

impl AsUrl for &UrlCow<'_> {
	#[inline]
	fn as_url(&self) -> Url<'_> { (**self).as_url() }
}

impl<'a, T: AsUrl> From<&'a T> for Url<'a> {
	fn from(value: &'a T) -> Self { value.as_url() }
}

impl<'a, T: AsUrl> From<&'a mut T> for Url<'a> {
	fn from(value: &'a mut T) -> Self { value.as_url() }
}
