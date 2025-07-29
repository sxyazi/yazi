use std::{hash::{Hash, Hasher}, ops::Deref};

use serde::{Deserialize, Serialize};

use crate::url::Url;

#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
#[repr(transparent)]
pub struct CovUrl(pub Url);

impl Deref for CovUrl {
	type Target = Url;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl AsRef<Url> for CovUrl {
	fn as_ref(&self) -> &Url { &self.0 }
}

impl From<Url> for CovUrl {
	fn from(value: Url) -> Self { Self(value) }
}

impl From<CovUrl> for Url {
	fn from(value: CovUrl) -> Self { value.0 }
}

impl Hash for CovUrl {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.loc.hash(state);
		if self.scheme.is_virtual() {
			self.scheme.hash(state);
		}
	}
}

impl PartialEq for CovUrl {
	fn eq(&self, other: &Self) -> bool { self.covariant(other) }
}

impl PartialEq<Url> for CovUrl {
	fn eq(&self, other: &Url) -> bool { self.covariant(other) }
}

impl CovUrl {
	#[inline]
	pub fn new<T: AsRef<Url>>(u: &T) -> &Self {
		unsafe { &*(u.as_ref() as *const Url as *const Self) }
	}

	#[inline]
	pub fn parent_url(&self) -> Option<CovUrl> { self.0.parent_url().map(CovUrl) }
}
