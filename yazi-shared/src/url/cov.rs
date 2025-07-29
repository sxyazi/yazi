use std::{hash::{Hash, Hasher}, ops::Deref};

use crate::url::Url;

#[derive(Clone, Debug, Eq)]
#[repr(transparent)]
pub struct CovUrl(pub Url);

impl Deref for CovUrl {
	type Target = Url;

	fn deref(&self) -> &Self::Target { &self.0 }
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

impl CovUrl {
	#[inline]
	pub fn new<T: AsRef<Url>>(u: &T) -> &Self {
		unsafe { &*(u.as_ref() as *const Url as *const Self) }
	}

	#[inline]
	pub fn parent_url(&self) -> Option<CovUrl> { self.0.parent_url().map(CovUrl) }
}
