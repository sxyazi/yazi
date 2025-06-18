use std::{any::Any, borrow::Cow, collections::HashMap};

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize, de};

use crate::{Id, url::{Url, UrnBuf}};

// --- Data
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Data {
	Nil,
	Boolean(bool),
	Integer(i64),
	Number(f64),
	String(Cow<'static, str>),
	List(Vec<Data>),
	Dict(HashMap<DataKey, Data>),
	Id(Id),
	#[serde(skip_deserializing)]
	Url(Url),
	#[serde(skip_deserializing)]
	Urn(UrnBuf),
	#[serde(skip)]
	Bytes(Vec<u8>),
	#[serde(skip)]
	Any(Box<dyn Any + Send + Sync>),
}

impl Data {
	#[inline]
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Self::Boolean(b) => Some(*b),
			Self::String(s) if s == "no" => Some(false),
			Self::String(s) if s == "yes" => Some(true),
			_ => None,
		}
	}

	#[inline]
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn into_string(self) -> Option<Cow<'static, str>> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn into_dict(self) -> Option<HashMap<DataKey, Data>> {
		match self {
			Self::Dict(d) => Some(d),
			_ => None,
		}
	}

	#[inline]
	pub fn into_url(self) -> Option<Url> {
		match self {
			Self::String(s) => Url::try_from(s.as_ref()).ok(),
			Self::Url(u) => Some(u),
			Self::Bytes(b) => Url::try_from(b.as_slice()).ok(),
			_ => None,
		}
	}

	#[inline]
	pub fn into_any<T: 'static>(self) -> Option<T> {
		match self {
			Self::Any(b) => b.downcast::<T>().ok().map(|b| *b),
			_ => None,
		}
	}

	#[inline]
	pub fn to_url(&self) -> Option<Url> {
		match self {
			Self::String(s) => Url::try_from(s.as_ref()).ok(),
			Self::Url(u) => Some(u.clone()),
			Self::Bytes(b) => Url::try_from(b.as_slice()).ok(),
			_ => None,
		}
	}
}

impl From<bool> for Data {
	fn from(value: bool) -> Self { Self::Boolean(value) }
}

impl From<i32> for Data {
	fn from(value: i32) -> Self { Self::Integer(value as i64) }
}

impl From<i64> for Data {
	fn from(value: i64) -> Self { Self::Integer(value) }
}

impl From<f64> for Data {
	fn from(value: f64) -> Self { Self::Number(value) }
}

impl From<usize> for Data {
	fn from(value: usize) -> Self { Self::Id(value.into()) }
}

impl From<String> for Data {
	fn from(value: String) -> Self { Self::String(Cow::Owned(value)) }
}

impl From<Cow<'static, str>> for Data {
	fn from(value: Cow<'static, str>) -> Self { Self::String(value) }
}

impl From<Id> for Data {
	fn from(value: Id) -> Self { Self::Id(value) }
}

impl From<&Url> for Data {
	fn from(value: &Url) -> Self { Self::Url(value.clone()) }
}

impl From<&str> for Data {
	fn from(value: &str) -> Self { Self::String(Cow::Owned(value.to_owned())) }
}

// --- Key
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataKey {
	Nil,
	Boolean(bool),
	#[serde(deserialize_with = "Self::deserialize_integer")]
	Integer(i64),
	Number(OrderedFloat<f64>),
	String(Cow<'static, str>),
	Id(Id),
	#[serde(skip_deserializing)]
	Url(Url),
	#[serde(skip_deserializing)]
	Urn(UrnBuf),
	#[serde(skip)]
	Bytes(Vec<u8>),
}

impl DataKey {
	#[inline]
	pub fn is_integer(&self) -> bool { matches!(self, Self::Integer(_)) }

	#[inline]
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn into_url(self) -> Option<Url> {
		match self {
			Self::String(s) => Url::try_from(s.as_ref()).ok(),
			Self::Url(u) => Some(u),
			Self::Bytes(b) => Url::try_from(b.as_slice()).ok(),
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

// --- Macros
macro_rules! impl_as_integer {
	($t:ty, $name:ident) => {
		impl Data {
			#[inline]
			pub fn $name(&self) -> Option<$t> {
				match self {
					Self::Integer(i) => <$t>::try_from(*i).ok(),
					Self::String(s) => s.parse().ok(),
					Self::Id(i) => <$t>::try_from(i.get()).ok(),
					_ => None,
				}
			}
		}
	};
}

macro_rules! impl_as_number {
	($t:ty, $name:ident) => {
		impl Data {
			#[inline]
			pub fn $name(&self) -> Option<$t> {
				match self {
					Self::Integer(i) if *i == (*i as $t as _) => Some(*i as $t),
					Self::Number(n) => <$t>::try_from(*n).ok(),
					Self::String(s) => s.parse().ok(),
					Self::Id(i) if i.0 == (i.0 as $t as _) => Some(i.0 as $t),
					_ => None,
				}
			}
		}
	};
}

impl_as_integer!(usize, as_usize);
impl_as_integer!(isize, as_isize);
impl_as_integer!(i16, as_i16);
impl_as_integer!(i32, as_i32);
impl_as_integer!(crate::Id, as_id);

impl_as_number!(f64, as_f64);
