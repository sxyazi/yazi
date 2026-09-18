use std::{path::{Path, PathBuf}, sync::Arc};

use mlua::{ExternalError, FromLua, Lua, Table, Value};
use serde::{Deserialize, Serialize};
use serde_with::{TryFromInto, serde_as};
use yazi_shared::path::PathBufDyn;

use crate::stat::{Stat, StatKind};

#[repr(transparent)]
#[derive(Clone, Debug, Default)]
pub struct FileExtra(Option<Arc<FileExtraInner>>);

#[serde_as]
#[derive(Debug, Default, Deserialize, Serialize)]
pub(super) struct FileExtraInner {
	pub(super) lstat:   Stat,
	pub(super) link_to: Option<PathBufDyn>,
	#[serde_as(as = "Option<TryFromInto<PathBufDyn>>")]
	pub(super) backing: Option<PathBuf>,
}

impl FileExtra {
	#[inline]
	pub fn new(lstat: Stat, link_to: Option<PathBufDyn>, backing: Option<PathBuf>) -> Self {
		Self(
			(lstat.is_link() || link_to.is_some() || backing.is_some())
				.then(|| Arc::new(FileExtraInner { lstat, link_to, backing })),
		)
	}

	#[inline]
	pub(super) fn as_ref(&self) -> Option<&FileExtraInner> { self.0.as_deref() }

	#[inline]
	pub(super) fn lstat(&self) -> Option<Stat> { Some(self.as_ref()?.lstat) }

	#[inline]
	pub fn link_to(&self) -> Option<&PathBufDyn> { self.as_ref()?.link_to.as_ref() }

	#[inline]
	pub(crate) fn backing(&self) -> Option<&Path> { self.as_ref()?.backing.as_deref() }
}

impl TryFrom<Table> for FileExtra {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let lstat: Stat = value.raw_get("lstat")?;
		if lstat.kind.contains(StatKind::FOLLOW) {
			return Err("File requires unfollowed metadata in `lstat`".into_lua_err());
		}

		Ok(Self::new(
			lstat,
			value.raw_get("link_to")?,
			value.raw_get::<Option<PathBufDyn>>("backing")?.map(PathBufDyn::into_os).transpose()?,
		))
	}
}

impl<'de> Deserialize<'de> for FileExtra {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let inner = FileExtraInner::deserialize(deserializer)?;
		Ok(Self::new(inner.lstat, inner.link_to, inner.backing))
	}
}

impl FromLua for FileExtra {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Table::from_lua(value, lua)?.try_into()
	}
}
