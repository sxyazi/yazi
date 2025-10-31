use std::{any::Any, borrow::Cow, path::PathBuf};

use anyhow::{Result, bail};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::{Id, SStr, data::DataKey, url::{UrlBuf, UrlCow}};

// --- Data
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Data {
	Nil,
	Boolean(bool),
	Integer(i64),
	Number(f64),
	String(SStr),
	List(Vec<Data>),
	Dict(HashMap<DataKey, Data>),
	Id(Id),
	#[serde(skip_deserializing)]
	Url(UrlBuf),
	#[serde(skip_deserializing)]
	Path(PathBuf),
	#[serde(skip)]
	Bytes(Vec<u8>),
	#[serde(skip)]
	Any(Box<dyn Any + Send + Sync>),
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
			Data::String(s) => Ok(s.as_ref().into()),
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
			Data::String(s) => s.as_ref().try_into(),
			Data::Url(u) => Ok(u.into()),
			Data::Bytes(b) => b.as_slice().try_into(),
			_ => bail!("not a URL"),
		}
	}
}

impl PartialEq<bool> for Data {
	fn eq(&self, other: &bool) -> bool { self.try_into().is_ok_and(|b| *other == b) }
}

// --- Macros
macro_rules! impl_into_integer {
	($t:ty, $name:ident) => {
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
	($t:ty, $name:ident) => {
		impl TryFrom<&Data> for $t {
			type Error = anyhow::Error;

			fn try_from(value: &Data) -> Result<Self, Self::Error> {
				Ok(match value {
					Data::Integer(i) if *i == (*i as $t as _) => *i as $t,
					Data::Number(n) => <$t>::try_from(*n)?,
					Data::String(s) => s.parse()?,
					Data::Id(i) if i.0 == (i.0 as $t as _) => i.0 as $t,
					_ => bail!("not a number"),
				})
			}
		}
	};
}

impl_into_integer!(usize, as_usize);
impl_into_integer!(isize, as_isize);
impl_into_integer!(i16, as_i16);
impl_into_integer!(i32, as_i32);
impl_into_integer!(crate::Id, as_id);

impl_into_number!(f64, as_f64);
