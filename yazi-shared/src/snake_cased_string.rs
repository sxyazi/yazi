use std::{borrow::Borrow, ffi::OsStr, fmt::{Display, Formatter}, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize};

use crate::BytesExt;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct SnakeCasedString(pub(super) String);

impl SnakeCasedString {
	pub fn new(s: String) -> Option<Self> { s.as_bytes().snake_cased().then_some(Self(s)) }
}

impl Deref for SnakeCasedString {
	type Target = str;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Borrow<str> for SnakeCasedString {
	#[inline]
	fn borrow(&self) -> &str { &self.0 }
}

impl Borrow<String> for SnakeCasedString {
	#[inline]
	fn borrow(&self) -> &String { &self.0 }
}

impl AsRef<str> for SnakeCasedString {
	#[inline]
	fn as_ref(&self) -> &str { &self.0 }
}

impl AsRef<OsStr> for SnakeCasedString {
	#[inline]
	fn as_ref(&self) -> &OsStr { self.0.as_ref() }
}

impl Display for SnakeCasedString {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Display::fmt(&self.0, f) }
}

impl From<SnakeCasedString> for String {
	#[inline]
	fn from(value: SnakeCasedString) -> Self { value.0 }
}

impl<'de> Deserialize<'de> for SnakeCasedString {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let value = String::deserialize(deserializer)?;
		Self::new(value).ok_or_else(|| serde::de::Error::custom("must be a snake-cased string"))
	}
}
