use std::{borrow::{Borrow, Cow}, ffi::OsStr, fmt::{Display, Formatter}, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize};

use crate::{BytesExt, SnakeCasedString};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct KebabCasedKey(String);

impl KebabCasedKey {
	pub fn new(s: String) -> Option<Self> {
		(!s.is_empty() && s.len() < 20 && s.as_bytes().kebab_cased()).then_some(Self(s))
	}

	pub fn into_snake_cased(self) -> SnakeCasedString {
		let mut b = self.0.into_bytes();
		b.iter_mut().for_each(|c| {
			if *c == b'-' {
				*c = b'_'
			}
		});
		SnakeCasedString(unsafe { String::from_utf8_unchecked(b) })
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

impl Borrow<String> for KebabCasedKey {
	#[inline]
	fn borrow(&self) -> &String { &self.0 }
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

impl From<KebabCasedKey> for String {
	#[inline]
	fn from(value: KebabCasedKey) -> Self { value.0 }
}

impl From<KebabCasedKey> for Cow<'_, str> {
	#[inline]
	fn from(value: KebabCasedKey) -> Self { Cow::Owned(value.0) }
}

impl<'de> Deserialize<'de> for KebabCasedKey {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let value = String::deserialize(deserializer)?;
		Self::new(value).ok_or_else(|| {
			serde::de::Error::custom("must be a non-empty kebab-cased key shorter than 20 characters")
		})
	}
}
