use std::{borrow::Borrow, ffi::{OsStr, OsString}, fmt::{Display, Formatter}, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct NonEmptyString(String);

impl NonEmptyString {
	#[inline]
	pub fn new(value: String) -> Option<Self> {
		Some(NonEmptyString(value)).filter(|s| !s.is_empty())
	}
}

impl Deref for NonEmptyString {
	type Target = str;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Borrow<str> for NonEmptyString {
	#[inline]
	fn borrow(&self) -> &str { &self.0 }
}

impl Borrow<String> for NonEmptyString {
	#[inline]
	fn borrow(&self) -> &String { &self.0 }
}

impl AsRef<str> for NonEmptyString {
	#[inline]
	fn as_ref(&self) -> &str { &self.0 }
}

impl AsRef<OsStr> for NonEmptyString {
	#[inline]
	fn as_ref(&self) -> &OsStr { self.0.as_ref() }
}

impl Display for NonEmptyString {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Display::fmt(&self.0, f) }
}

impl From<NonEmptyString> for String {
	#[inline]
	fn from(value: NonEmptyString) -> Self { value.0 }
}

impl From<NonEmptyString> for OsString {
	#[inline]
	fn from(value: NonEmptyString) -> Self { value.0.into() }
}

impl<'de> Deserialize<'de> for NonEmptyString {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let value = String::deserialize(deserializer)?;
		Self::new(value).ok_or_else(|| serde::de::Error::custom("must be a non-empty string"))
	}
}
