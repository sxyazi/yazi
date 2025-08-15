use std::{hash::{Hash, Hasher}, ops::Deref};

use serde::{Deserialize, Serialize};

use crate::url::UrlBuf;

#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
#[repr(transparent)]
pub struct UrlCov(pub UrlBuf);

impl Deref for UrlCov {
	type Target = UrlBuf;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl AsRef<UrlBuf> for UrlCov {
	fn as_ref(&self) -> &UrlBuf { &self.0 }
}

impl From<UrlBuf> for UrlCov {
	fn from(value: UrlBuf) -> Self { Self(value) }
}

impl From<UrlCov> for UrlBuf {
	fn from(value: UrlCov) -> Self { value.0 }
}

impl Hash for UrlCov {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.loc.hash(state);
		if self.scheme.is_virtual() {
			self.scheme.hash(state);
		}
	}
}

impl PartialEq for UrlCov {
	fn eq(&self, other: &Self) -> bool { self.covariant(other) }
}

impl PartialEq<UrlBuf> for UrlCov {
	fn eq(&self, other: &UrlBuf) -> bool { self.covariant(other) }
}

impl UrlCov {
	#[inline]
	pub fn new<T: AsRef<UrlBuf>>(u: &T) -> &Self {
		unsafe { &*(u.as_ref() as *const UrlBuf as *const Self) }
	}

	#[inline]
	pub fn parent_url(&self) -> Option<UrlCov> { self.0.parent_url().map(UrlCov) }
}
