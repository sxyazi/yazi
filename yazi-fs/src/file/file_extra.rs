use std::{path::{Path, PathBuf}, sync::Arc};

use mlua::{FromLua, Lua, Table, Value};
use serde::{Deserialize, Serialize};
use serde_with::{TryFromInto, serde_as};
use yazi_shared::path::PathBufDyn;

#[repr(transparent)]
#[derive(Clone, Debug, Default)]
pub struct FileExtra(Option<Arc<FileExtraInner>>);

#[serde_as]
#[derive(Debug, Default, Deserialize, Serialize)]
struct FileExtraInner {
	link_to: Option<PathBufDyn>,
	#[serde_as(as = "Option<TryFromInto<PathBufDyn>>")]
	backing: Option<PathBuf>,
}

impl FileExtra {
	#[inline]
	pub fn new(link_to: Option<PathBufDyn>, backing: Option<PathBuf>) -> Self {
		Self(
			(link_to.is_some() || backing.is_some())
				.then(|| Arc::new(FileExtraInner { link_to, backing })),
		)
	}

	#[inline]
	pub fn link_to(&self) -> Option<&PathBufDyn> { self.0.as_ref()?.link_to.as_ref() }

	#[inline]
	pub fn backing(&self) -> Option<&Path> { self.0.as_ref()?.backing.as_deref() }
}

impl TryFrom<Table> for FileExtra {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		Ok(Self::new(
			value.raw_get("link_to")?,
			value.raw_get::<Option<PathBufDyn>>("backing")?.map(PathBufDyn::into_os).transpose()?,
		))
	}
}

impl Serialize for FileExtra {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		match &self.0 {
			Some(inner) => inner.serialize(serializer),
			None => FileExtraInner::default().serialize(serializer),
		}
	}
}

impl<'de> Deserialize<'de> for FileExtra {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let inner = FileExtraInner::deserialize(deserializer)?;
		Ok(Self::new(inner.link_to, inner.backing))
	}
}

impl FromLua for FileExtra {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Table::from_lua(value, lua)?.try_into()
	}
}
