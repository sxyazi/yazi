use std::{borrow::Borrow, ffi::OsStr, fmt::{Display, Formatter}, ops::Deref};

use compact_str::CompactString;
use serde::{Deserialize, Deserializer, Serialize};

use crate::BytesExt;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct SnakeCasedKey(pub(super) CompactString);

impl SnakeCasedKey {
	pub fn new(s: impl Into<CompactString>) -> Option<Self> {
		let s = s.into();
		(!s.is_empty() && s.len() <= 20 && s.as_bytes().snake_cased()).then_some(Self(s))
	}
}

impl Deref for SnakeCasedKey {
	type Target = str;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Borrow<str> for SnakeCasedKey {
	#[inline]
	fn borrow(&self) -> &str { &self.0 }
}

impl AsRef<str> for SnakeCasedKey {
	#[inline]
	fn as_ref(&self) -> &str { &self.0 }
}

impl AsRef<OsStr> for SnakeCasedKey {
	#[inline]
	fn as_ref(&self) -> &OsStr { self.0.as_ref() }
}

impl Display for SnakeCasedKey {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Display::fmt(&self.0, f) }
}

impl<'de> Deserialize<'de> for SnakeCasedKey {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let value = CompactString::deserialize(deserializer)?;
		Self::new(value)
			.ok_or_else(|| serde::de::Error::custom("must be 1-20 characters in snake-case"))
	}
}
