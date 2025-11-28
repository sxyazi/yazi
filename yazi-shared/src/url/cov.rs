use std::{hash::{Hash, Hasher}, ops::Deref, path::PathBuf};

use hashbrown::Equivalent;
use serde::{Deserialize, Serialize};

use crate::url::{AsUrl, Url, UrlBuf, UrlCow, UrlLike};

#[derive(Clone, Copy)]
pub struct UrlCov<'a>(Url<'a>);

impl<'a> Deref for UrlCov<'a> {
	type Target = Url<'a>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'a> From<&'a UrlBufCov> for UrlCov<'a> {
	fn from(value: &'a UrlBufCov) -> Self { Self(value.0.as_url()) }
}

impl PartialEq<UrlBufCov> for UrlCov<'_> {
	fn eq(&self, other: &UrlBufCov) -> bool { self.0.covariant(&other.0) }
}

impl Hash for UrlCov<'_> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.loc().hash(state);
		if self.0.kind().is_virtual() {
			self.0.scheme().hash(state);
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

impl From<UrlBufCov> for UrlBuf {
	fn from(value: UrlBufCov) -> Self { value.0 }
}

impl From<&UrlBufCov> for UrlBuf {
	fn from(value: &UrlBufCov) -> Self { value.0.clone() }
}

impl From<UrlBuf> for UrlBufCov {
	fn from(value: UrlBuf) -> Self { Self(value) }
}

impl From<PathBuf> for UrlBufCov {
	fn from(value: PathBuf) -> Self { Self(UrlBuf::from(value)) }
}

impl From<UrlCow<'_>> for UrlBufCov {
	fn from(value: UrlCow<'_>) -> Self { Self(value.into_owned()) }
}

impl From<Url<'_>> for UrlBufCov {
	fn from(value: Url<'_>) -> Self { Self(value.to_owned()) }
}

impl From<&UrlCov<'_>> for UrlBufCov {
	fn from(value: &UrlCov<'_>) -> Self { Self(UrlBuf::from(&value.0)) }
}

impl<'a> From<&'a UrlBufCov> for Url<'a> {
	fn from(value: &'a UrlBufCov) -> Self { value.0.as_url() }
}

impl Hash for UrlBufCov {
	fn hash<H: Hasher>(&self, state: &mut H) { UrlCov::from(self).hash(state) }
}

impl PartialEq for UrlBufCov {
	fn eq(&self, other: &Self) -> bool { self.covariant(&other.0) }
}

impl PartialEq<UrlBuf> for UrlBufCov {
	fn eq(&self, other: &UrlBuf) -> bool { self.covariant(other) }
}
