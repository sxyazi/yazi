use std::{borrow::Borrow, ffi::OsStr, fmt::{Display, Formatter}, ops::Deref};

use compact_str::CompactString;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{BytesExt, SnakeCasedKey};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct KebabCasedKey(CompactString);

impl KebabCasedKey {
	pub(crate) fn new(s: impl Into<CompactString>) -> Option<Self> {
		let s = s.into();
		(!s.is_empty() && s.len() <= 20 && s.as_bytes().kebab_cased()).then_some(Self(s))
	}

	pub fn into_snake_cased(self) -> SnakeCasedKey {
		SnakeCasedKey(self.0.chars().map(|c| if c == '-' { '_' } else { c }).collect())
	}
}

impl Deref for KebabCasedKey {
	type Target = str;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Borrow<str> for KebabCasedKey {
	#[inline]
	fn borrow(&self) -> &str { &self.0 }
}

impl AsRef<str> for KebabCasedKey {
	#[inline]
	fn as_ref(&self) -> &str { &self.0 }
}

impl AsRef<OsStr> for KebabCasedKey {
	#[inline]
	fn as_ref(&self) -> &OsStr { self.0.as_ref() }
}

impl Display for KebabCasedKey {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Display::fmt(&self.0, f) }
}

impl<'de> Deserialize<'de> for KebabCasedKey {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let value = CompactString::deserialize(deserializer)?;
		Self::new(value)
			.ok_or_else(|| serde::de::Error::custom("must be 1-20 characters in kebab-case"))
	}
}
