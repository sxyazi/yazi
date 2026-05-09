use std::borrow::Cow;

use anyhow::{Result, anyhow, bail};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::{Id, SStr, data::{DataAny, DataKey}, path::PathBufDyn, strand::{IntoStrand, StrandBuf}, url::{Url, UrlBuf, UrlCow}};

// --- Data
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Data {
	#[default]
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
	Any(Box<dyn DataAny>),
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

impl From<Url<'_>> for Data {
	fn from(value: Url) -> Self { Self::Url(value.into()) }
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

impl<T> FromIterator<T> for Data
where
	T: Into<Self>,
{
	fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
		Self::List(iter.into_iter().map(Into::into).collect())
	}
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
		value.into_sstr().ok_or_else(|| anyhow!("not a string"))
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

impl TryFrom<&Data> for String {
	type Error = anyhow::Error;

	fn try_from(value: &Data) -> Result<Self, Self::Error> {
		value.try_into().map(|s: &str| s.to_owned())
	}
}

impl TryFrom<Data> for String {
	type Error = anyhow::Error;

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		SStr::try_from(value).map(|s| s.into_owned())
	}
}

impl TryFrom<&Data> for HashMap<DataKey, Data> {
	type Error = anyhow::Error;

	fn try_from(_: &Data) -> Result<Self, Self::Error> {
		bail!("cannot take ownership of dict from &Data");
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

impl TryFrom<Data> for UrlBuf {
	type Error = anyhow::Error;

	fn try_from(value: Data) -> Result<Self, Self::Error> { UrlCow::try_from(value).map(Into::into) }
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

impl TryFrom<&Data> for UrlBuf {
	type Error = anyhow::Error;

	fn try_from(value: &Data) -> Result<Self, Self::Error> { UrlCow::try_from(value).map(Into::into) }
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

impl TryFrom<Data> for StrandBuf {
	type Error = anyhow::Error;

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		Ok(match value {
			Data::String(s) => s.into_owned().into(),
			Data::Path(p) => p.into_strand(),
			Data::Bytes(b) => Self::Bytes(b),
			_ => bail!("cannot convert to StrandBuf"),
		})
	}
}

impl TryFrom<&Data> for StrandBuf {
	type Error = anyhow::Error;

	fn try_from(value: &Data) -> Result<Self, Self::Error> {
		Ok(match value {
			Data::String(s) => s.to_string().into(),
			Data::Path(p) => p.into_strand(),
			Data::Bytes(b) => Self::Bytes(b.clone()),
			_ => bail!("cannot convert to StrandBuf"),
		})
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

	pub fn as_any<T: 'static>(&self) -> Option<&T> {
		match self {
			Self::Any(b) => (**b).as_any().downcast_ref::<T>(),
			_ => None,
		}
	}

	pub fn into_sstr(self) -> Option<SStr> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	pub fn into_any<T: 'static>(self) -> Option<T> {
		match self {
			Self::Any(b) => b.into_any().downcast::<T>().ok().map(|b| *b),
			_ => None,
		}
	}

	// FIXME: find a better name
	pub fn into_any2<T: 'static>(self) -> Result<T> {
		if let Self::Any(b) = self
			&& let Ok(t) = b.into_any().downcast::<T>()
		{
			Ok(*t)
		} else {
			bail!("Failed to downcast Data into {}", std::any::type_name::<T>())
		}
	}
}

impl_into_integer!(Data, i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, crate::Id);
impl_into_number!(Data, f32, f64);
