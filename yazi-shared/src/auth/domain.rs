use std::{borrow::{Borrow, Cow}, fmt::{self, Display, Formatter}, ops::Deref, str};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::{self, IntoDeserializer, Visitor}};

use crate::{BytesExt, data::BytesDeserializer};

#[derive(Default, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Domain<'a>(Cow<'a, [u8]>);

impl Domain<'static> {
	pub const CATCHALL: Self = Self(Cow::Borrowed(b"*"));
	pub const EMPTY: Self = Self(Cow::Borrowed(b""));
}

impl Deref for Domain<'_> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Borrow<[u8]> for Domain<'_> {
	fn borrow(&self) -> &[u8] { &self.0 }
}

impl AsRef<[u8]> for Domain<'_> {
	fn as_ref(&self) -> &[u8] { &self.0 }
}

impl<'a> From<Vec<u8>> for Domain<'a> {
	fn from(value: Vec<u8>) -> Self { Self(Cow::Owned(value)) }
}

impl<'a> From<Cow<'a, [u8]>> for Domain<'a> {
	fn from(value: Cow<'a, [u8]>) -> Self { Self(value) }
}

impl<'a> From<String> for Domain<'a> {
	fn from(value: String) -> Self { Self(Cow::Owned(value.into_bytes())) }
}

impl<'a, T> From<&'a T> for Domain<'a>
where
	T: AsRef<[u8]> + ?Sized,
{
	fn from(value: &'a T) -> Self { Self(Cow::Borrowed(value.as_ref())) }
}

impl Display for Domain<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { self.0.display().fmt(f) }
}

impl<'a> Domain<'a> {
	pub fn into_owned(self) -> Domain<'static> { Domain(Cow::Owned(self.0.into_owned())) }

	pub fn is_catchall(&self) -> bool { *self == Domain::CATCHALL }

	pub fn to_str(&self) -> Result<&str, str::Utf8Error> { str::from_utf8(&self.0) }
}

impl<'de, 'a: 'de> IntoDeserializer<'de, de::value::Error> for Domain<'a> {
	type Deserializer = BytesDeserializer<'a>;

	fn into_deserializer(self) -> Self::Deserializer { BytesDeserializer(self.0) }
}

impl Serialize for Domain<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		match self.to_str() {
			Ok(s) => serializer.serialize_str(s),
			Err(_) => serializer.serialize_bytes(&self.0),
		}
	}
}

impl<'de> Deserialize<'de> for Domain<'_> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		struct V;

		impl Visitor<'_> for V {
			type Value = Domain<'static>;

			fn expecting(&self, f: &mut Formatter) -> fmt::Result { f.write_str("a string or bytes") }

			fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
				Ok(v.to_owned().into())
			}

			fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> { Ok(v.into()) }

			fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
				Ok(v.to_owned().into())
			}

			fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> { Ok(v.into()) }
		}

		deserializer.deserialize_any(V)
	}
}
