use std::{any::Any, borrow::Cow};

use anyhow::{Result, bail};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::{Id, SStr, data::DataKey, path::PathBufDyn, url::{UrlBuf, UrlCow}};

// --- Data
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Data {
	Nil,
	Boolean(bool),
	Integer(i64),
	Number(f64),
	String(SStr),
	List(Vec<Self>),
	Dict(HashMap<DataKey, Self>),
	Id(Id),
	#[serde(skip_deserializing)]
	Url(UrlBuf),
	#[serde(skip)]
	Path(PathBufDyn),
	#[serde(skip)]
	Bytes(Vec<u8>),
	#[serde(skip)]
	Any(Box<dyn Any + Send + Sync>),
}

impl From<()> for Data {
	fn from(_: ()) -> Self { Self::Nil }
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

impl From<SStr> for Data {
	fn from(value: SStr) -> Self { Self::String(value) }
}

impl From<Id> for Data {
	fn from(value: Id) -> Self { Self::Id(value) }
}

impl From<UrlBuf> for Data {
	fn from(value: UrlBuf) -> Self { Self::Url(value) }
}

impl From<&UrlBuf> for Data {
	fn from(value: &UrlBuf) -> Self { Self::Url(value.clone()) }
}

impl From<&str> for Data {
	fn from(value: &str) -> Self { Self::String(Cow::Owned(value.to_owned())) }
}

impl TryFrom<&Data> for bool {
	type Error = anyhow::Error;

	fn try_from(value: &Data) -> Result<Self, Self::Error> {
		match value {
			Data::Boolean(b) => Ok(*b),
			Data::String(s) if s == "no" => Ok(false),
			Data::String(s) if s == "yes" => Ok(true),
			_ => bail!("not a boolean"),
		}
	}
}

impl<'a> TryFrom<&'a Data> for &'a str {
	type Error = anyhow::Error;

	fn try_from(value: &'a Data) -> Result<Self, Self::Error> {
		match value {
			Data::String(s) => Ok(s),
			_ => bail!("not a string"),
		}
	}
}

impl TryFrom<Data> for SStr {
	type Error = anyhow::Error;

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		match value {
			Data::String(s) => Ok(s),
			_ => bail!("not a string"),
		}
	}
}

impl<'a> TryFrom<&'a Data> for Cow<'a, str> {
	type Error = anyhow::Error;

	fn try_from(value: &'a Data) -> Result<Self, Self::Error> {
		match value {
			Data::String(s) => Ok(Cow::Borrowed(s)),
			_ => bail!("not a string"),
		}
	}
}

impl TryFrom<Data> for HashMap<DataKey, Data> {
	type Error = anyhow::Error;

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		match value {
			Data::Dict(d) => Ok(d),
			_ => bail!("not a dict"),
		}
	}
}

impl TryFrom<&Data> for HashMap<DataKey, Data> {
	type Error = anyhow::Error;

	fn try_from(_: &Data) -> Result<Self, Self::Error> {
		bail!("cannot take ownership of dict from &Data");
	}
}

impl TryFrom<Data> for UrlCow<'static> {
	type Error = anyhow::Error;

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		match value {
			Data::String(s) => s.try_into(),
			Data::Url(u) => Ok(u.into()),
			Data::Bytes(b) => b.try_into(),
			_ => bail!("not a URL"),
		}
	}
}

impl<'a> TryFrom<&'a Data> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from(value: &'a Data) -> Result<Self, Self::Error> {
		match value {
			Data::String(s) => Self::try_from(&**s),
			Data::Url(u) => Ok(u.into()),
			Data::Bytes(b) => b.as_slice().try_into(),
			_ => bail!("not a URL"),
		}
	}
}

impl<'a> TryFrom<&'a Data> for &'a [u8] {
	type Error = anyhow::Error;

	fn try_from(value: &'a Data) -> Result<Self, Self::Error> {
		match value {
			Data::Bytes(b) => Ok(b),
			_ => bail!("not bytes"),
		}
	}
}

impl PartialEq<bool> for Data {
	fn eq(&self, other: &bool) -> bool { self.try_into().is_ok_and(|b| *other == b) }
}

impl Data {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	pub fn into_string(self) -> Option<SStr> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	pub fn into_any<T: 'static>(self) -> Option<T> {
		match self {
			Self::Any(b) => b.downcast::<T>().ok().map(|b| *b),
			_ => None,
		}
	}

	// FIXME: find a better name
	pub fn into_any2<T: 'static>(self) -> Result<T> {
		if let Self::Any(b) = self
			&& let Ok(t) = b.downcast::<T>()
		{
			Ok(*t)
		} else {
			bail!("Failed to downcast Data into {}", std::any::type_name::<T>())
		}
	}
}

impl<'de> serde::Deserializer<'de> for &Data {
	type Error = serde::de::value::Error;

	fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(serde::de::Error::custom("any not supported"))
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_bool(self.try_into().map_err(serde::de::Error::custom)?)
	}

	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i8(self.try_into().map_err(serde::de::Error::custom)?)
	}

	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i16(self.try_into().map_err(serde::de::Error::custom)?)
	}

	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i32(self.try_into().map_err(serde::de::Error::custom)?)
	}

	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i64(self.try_into().map_err(serde::de::Error::custom)?)
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u8(self.try_into().map_err(serde::de::Error::custom)?)
	}

	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u16(self.try_into().map_err(serde::de::Error::custom)?)
	}

	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u32(self.try_into().map_err(serde::de::Error::custom)?)
	}

	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u64(self.try_into().map_err(serde::de::Error::custom)?)
	}

	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_f32(self.try_into().map_err(serde::de::Error::custom)?)
	}

	fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_f64(self.try_into().map_err(serde::de::Error::custom)?)
	}

	fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(serde::de::Error::custom("not a char"))
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_str(self.try_into().map_err(serde::de::Error::custom)?)
	}

	fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(serde::de::Error::custom("string not supported"))
	}

	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_bytes(self.try_into().map_err(serde::de::Error::custom)?)
	}

	fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(serde::de::Error::custom("byte buf not supported"))
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self {
			Data::Nil => visitor.visit_none(),
			_ => visitor.visit_some(self),
		}
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_unit()
	}

	fn deserialize_unit_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_unit()
	}

	fn deserialize_newtype_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(serde::de::Error::custom("seq not supported"))
	}

	fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(serde::de::Error::custom("tuple not supported"))
	}

	fn deserialize_tuple_struct<V>(
		self,
		_name: &'static str,
		_len: usize,
		_visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(serde::de::Error::custom("tuple struct not supported"))
	}

	fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(serde::de::Error::custom("map not supported"))
	}

	fn deserialize_struct<V>(
		self,
		_name: &'static str,
		_fields: &'static [&'static str],
		_visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(serde::de::Error::custom("struct not supported"))
	}

	fn deserialize_enum<V>(
		self,
		_name: &'static str,
		_variants: &'static [&'static str],
		_visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(serde::de::Error::custom("enum not supported"))
	}

	fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(serde::de::Error::custom("identifier not supported"))
	}

	fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Err(serde::de::Error::custom("ignored any not supported"))
	}
}

// --- Macros
macro_rules! impl_into_integer {
	($t:ty) => {
		impl TryFrom<&Data> for $t {
			type Error = anyhow::Error;

			fn try_from(value: &Data) -> Result<Self, Self::Error> {
				Ok(match value {
					Data::Integer(i) => <$t>::try_from(*i)?,
					Data::String(s) => s.parse()?,
					Data::Id(i) => <$t>::try_from(i.get())?,
					_ => bail!("not an integer"),
				})
			}
		}
	};
}

macro_rules! impl_into_number {
	($t:ty) => {
		impl TryFrom<&Data> for $t {
			type Error = anyhow::Error;

			fn try_from(value: &Data) -> Result<Self, Self::Error> {
				Ok(match value {
					Data::Integer(i) if *i == (*i as $t as _) => *i as $t,
					Data::Number(n) if *n == (*n as $t as _) => *n as $t,
					Data::String(s) => s.parse()?,
					Data::Id(i) if i.0 == (i.0 as $t as _) => i.0 as $t,
					_ => bail!("not a number"),
				})
			}
		}
	};
}

impl_into_integer!(i8);
impl_into_integer!(i16);
impl_into_integer!(i32);
impl_into_integer!(i64);
impl_into_integer!(isize);
impl_into_integer!(u8);
impl_into_integer!(u16);
impl_into_integer!(u32);
impl_into_integer!(u64);
impl_into_integer!(usize);
impl_into_integer!(crate::Id);

impl_into_number!(f32);
impl_into_number!(f64);
