use std::borrow::Cow;

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize, de};

use crate::{Id, SStr, path::PathBufDyn, url::{UrlBuf, UrlCow}};

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DataKey {
	Nil,
	Boolean(bool),
	#[serde(deserialize_with = "Self::deserialize_integer")]
	Integer(i64),
	Number(OrderedFloat<f64>),
	String(SStr),
	Id(Id),
	#[serde(skip_deserializing)]
	Url(UrlBuf),
	#[serde(skip)]
	Path(PathBufDyn),
	#[serde(skip)]
	Bytes(Vec<u8>),
}

impl DataKey {
	pub fn is_integer(&self) -> bool { matches!(self, Self::Integer(_)) }

	pub fn as_str(&self) -> Option<&str> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	pub fn into_url(self) -> Option<UrlCow<'static>> {
		match self {
			Self::String(s) => s.try_into().ok(),
			Self::Url(u) => Some(u.into()),
			Self::Bytes(b) => b.try_into().ok(),
			_ => None,
		}
	}

	fn deserialize_integer<'de, D>(deserializer: D) -> Result<i64, D::Error>
	where
		D: de::Deserializer<'de>,
	{
		struct Visitor;

		impl de::Visitor<'_> for Visitor {
			type Value = i64;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("an integer or a string of an integer")
			}

			fn visit_i64<E>(self, value: i64) -> Result<i64, E> { Ok(value) }

			fn visit_str<E>(self, value: &str) -> Result<i64, E>
			where
				E: de::Error,
			{
				value.parse().map_err(de::Error::custom)
			}
		}

		deserializer.deserialize_any(Visitor)
	}
}

impl From<usize> for DataKey {
	fn from(value: usize) -> Self { Self::Integer(value as i64) }
}

impl From<&'static str> for DataKey {
	fn from(value: &'static str) -> Self { Self::String(Cow::Borrowed(value)) }
}

impl From<String> for DataKey {
	fn from(value: String) -> Self { Self::String(Cow::Owned(value)) }
}
