use std::{any::Any, borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize, de};

use crate::{Id, OrderedFloat, url::{Url, UrnBuf}};

// --- Data
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Data {
	Nil,
	Boolean(bool),
	Integer(i64),
	Number(f64),
	String(String),
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
	pub fn into_any<T: 'static>(self) -> Option<T> {
		match self {
			Self::Any(b) => b.downcast::<T>().ok().map(|b| *b),
			_ => None,
		}
	}

	#[inline]
	pub fn into_url(self) -> Option<Url> {
		match self {
			Data::String(s) => Some(Url::from(s)),
			Data::Url(u) => Some(u),
			_ => None,
		}
	}

	pub fn into_dict_string(self) -> HashMap<Cow<'static, str>, String> {
		let Self::Dict(dict) = self else {
			return Default::default();
		};

		let mut map = HashMap::with_capacity(dict.len());
		for pair in dict {
			if let (DataKey::String(k), Self::String(v)) = pair {
				map.insert(k, v);
			}
		}
		map
	}

	#[inline]
	pub fn to_url(&self) -> Option<Url> {
		match self {
			Self::String(s) => Some(Url::from(s)),
			Self::Url(u) => Some(u.clone()),
			_ => None,
		}
	}
}

impl From<bool> for Data {
	fn from(value: bool) -> Self { Self::Boolean(value) }
}

impl From<usize> for Data {
	fn from(value: usize) -> Self { Self::Id(value.into()) }
}

impl From<String> for Data {
	fn from(value: String) -> Self { Self::String(value) }
}

impl From<Id> for Data {
	fn from(value: Id) -> Self { Self::Id(value) }
}

// --- Key
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataKey {
	Nil,
	Boolean(bool),
	#[serde(deserialize_with = "Self::deserialize_integer")]
	Integer(i64),
	Number(OrderedFloat),
	String(Cow<'static, str>),
	Id(Id),
	#[serde(skip_deserializing)]
	Url(Url),
	#[serde(skip_deserializing)]
	Urn(UrnBuf),
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
macro_rules! impl_integer_as {
	($t:ty, $name:ident) => {
		impl Data {
			#[inline]
			pub fn $name(&self) -> Option<$t> {
				match self {
					Data::Integer(i) => <$t>::try_from(*i).ok(),
					Data::String(s) => s.parse().ok(),
					Data::Id(i) => <$t>::try_from(i.get()).ok(),
					_ => None,
				}
			}
		}
	};
}

macro_rules! impl_number_as {
	($t:ty, $name:ident) => {
		impl Data {
			#[inline]
			pub fn $name(&self) -> Option<$t> {
				match self {
					Data::Number(n) => <$t>::try_from(*n).ok(),
					Data::String(s) => s.parse().ok(),
					_ => None,
				}
			}
		}
	};
}

impl_integer_as!(usize, as_usize);
impl_integer_as!(isize, as_isize);
impl_integer_as!(i16, as_i16);
impl_integer_as!(i32, as_i32);

impl_number_as!(f64, as_f64);

impl_integer_as!(crate::Id, as_id);
