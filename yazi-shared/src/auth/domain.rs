use std::{borrow::{Borrow, Cow}, fmt::{self, Display, Formatter}, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize, de::{self, IntoDeserializer, value::CowStrDeserializer}};

#[derive(Default, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Domain<'a>(Cow<'a, str>);

impl Domain<'static> {
	pub const CATCHALL: Self = Self(Cow::Borrowed("*"));
	pub const EMPTY: Self = Self(Cow::Borrowed(""));
}

impl Domain<'_> {
	#[inline]
	pub fn into_owned(self) -> Domain<'static> { Domain(Cow::Owned(self.0.into_owned())) }

	#[inline]
	pub fn is_catchall(&self) -> bool { *self == Domain::CATCHALL }
}

impl Deref for Domain<'_> {
	type Target = str;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Borrow<str> for Domain<'_> {
	fn borrow(&self) -> &str { &self.0 }
}

impl AsRef<str> for Domain<'_> {
	fn as_ref(&self) -> &str { &self.0 }
}

impl<'a> From<String> for Domain<'a> {
	fn from(value: String) -> Self { Self(Cow::Owned(value)) }
}

impl<'a> From<&'a str> for Domain<'a> {
	fn from(value: &'a str) -> Self { Self(Cow::Borrowed(value)) }
}

impl<'a> From<Cow<'a, str>> for Domain<'a> {
	fn from(value: Cow<'a, str>) -> Self { Self(value) }
}

impl<'a> From<&'a Domain<'_>> for Domain<'a> {
	fn from(value: &'a Domain<'_>) -> Self { Self(Cow::Borrowed(&value.0)) }
}

impl Display for Domain<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { Display::fmt(&self.0, f) }
}

impl<'de> Deserialize<'de> for Domain<'_> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		Ok(Self(Cow::Owned(String::deserialize(deserializer)?)))
	}
}

impl<'de, 'a, E> IntoDeserializer<'de, E> for Domain<'a>
where
	E: de::Error,
{
	type Deserializer = CowStrDeserializer<'a, E>;

	fn into_deserializer(self) -> Self::Deserializer { self.0.into_deserializer() }
}
