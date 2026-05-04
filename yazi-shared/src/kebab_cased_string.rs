use std::{borrow::{Borrow, Cow}, ffi::OsStr, fmt::{Display, Formatter}, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize};

use crate::{BytesExt, SnakeCasedString};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct KebabCasedString(String);

impl KebabCasedString {
	pub fn new(s: String) -> Option<Self> { s.as_bytes().kebab_cased().then_some(Self(s)) }

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

impl Deref for KebabCasedString {
	type Target = str;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Borrow<str> for KebabCasedString {
	#[inline]
	fn borrow(&self) -> &str { &self.0 }
}

impl Borrow<String> for KebabCasedString {
	#[inline]
	fn borrow(&self) -> &String { &self.0 }
}

impl AsRef<str> for KebabCasedString {
	#[inline]
	fn as_ref(&self) -> &str { &self.0 }
}

impl AsRef<OsStr> for KebabCasedString {
	#[inline]
	fn as_ref(&self) -> &OsStr { self.0.as_ref() }
}

impl Display for KebabCasedString {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Display::fmt(&self.0, f) }
}

impl From<KebabCasedString> for String {
	#[inline]
	fn from(value: KebabCasedString) -> Self { value.0 }
}

impl From<KebabCasedString> for Cow<'_, str> {
	#[inline]
	fn from(value: KebabCasedString) -> Self { Cow::Owned(value.0) }
}

impl<'de> Deserialize<'de> for KebabCasedString {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let value = String::deserialize(deserializer)?;
		Self::new(value).ok_or_else(|| serde::de::Error::custom("must be a kebab-cased string"))
	}
}
