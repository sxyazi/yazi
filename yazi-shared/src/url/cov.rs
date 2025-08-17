use std::{hash::{Hash, Hasher}, ops::Deref};

use hashbrown::Equivalent;
use serde::{Deserialize, Serialize};

use crate::url::{Url, UrlBuf};

#[derive(Clone)]
pub struct UrlCov<'a>(Url<'a>);

impl<'a> Deref for UrlCov<'a> {
	type Target = Url<'a>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'a> From<&'a UrlBufCov> for UrlCov<'a> {
	fn from(value: &'a UrlBufCov) -> Self { Self(value.0.as_url()) }
}

impl PartialEq<UrlBufCov> for UrlCov<'_> {
	fn eq(&self, other: &UrlBufCov) -> bool { self.0.covariant(other.0.as_url()) }
}

impl Hash for UrlCov<'_> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.loc.hash(state);
		if self.0.scheme.is_virtual() {
			self.0.scheme.hash(state);
		}
	}
}

impl Equivalent<UrlBufCov> for UrlCov<'_> {
	fn equivalent(&self, key: &UrlBufCov) -> bool { self == key }
}

impl<'a> UrlCov<'a> {
	#[inline]
	pub fn new(url: impl Into<Url<'a>>) -> Self { Self(url.into()) }
}

// --- Buf
#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
pub struct UrlBufCov(pub UrlBuf);

impl Deref for UrlBufCov {
	type Target = UrlBuf;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<UrlBuf> for UrlBufCov {
	fn from(value: UrlBuf) -> Self { Self(value) }
}

impl From<&UrlCov<'_>> for UrlBufCov {
	fn from(value: &UrlCov<'_>) -> Self { Self(UrlBuf::from(&value.0)) }
}

impl From<UrlBufCov> for UrlBuf {
	fn from(value: UrlBufCov) -> Self { value.0 }
}

impl<'a> From<&'a UrlBufCov> for Url<'a> {
	fn from(value: &'a UrlBufCov) -> Self { value.0.as_url() }
}

impl Hash for UrlBufCov {
	fn hash<H: Hasher>(&self, state: &mut H) { self.as_url().hash(state) }
}

impl PartialEq for UrlBufCov {
	fn eq(&self, other: &Self) -> bool { self.covariant(other) }
}

impl PartialEq<UrlBuf> for UrlBufCov {
	fn eq(&self, other: &UrlBuf) -> bool { self.covariant(other) }
}

impl UrlBufCov {
	#[inline]
	pub fn as_url(&self) -> UrlCov<'_> { UrlCov::from(self) }

	#[inline]
	pub fn parent_url(&self) -> Option<UrlBufCov> { self.0.parent_url().map(UrlBufCov) }
}
