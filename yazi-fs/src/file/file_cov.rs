use std::{hash::{Hash, Hasher}, ops::Deref};

use hashbrown::Equivalent;
use mlua::{FromLua, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::url::{UrlBuf, UrlBufCov, UrlCov};

use crate::file::File;

/// A newtype around [`File`] that hashes and compares by URL only
/// (covariantly), via [`UrlCov`]. Useful as a key in [`IndexMap`] / [`HashMap`]
/// when the file's metadata should not affect identity.
#[derive(Clone, Debug, Default)]
pub struct FileCov(pub File);

impl Deref for FileCov {
	type Target = File;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<File> for FileCov {
	fn from(value: File) -> Self { Self(value) }
}

impl From<&File> for FileCov {
	fn from(value: &File) -> Self { Self(value.clone()) }
}

impl From<FileCov> for File {
	fn from(value: FileCov) -> Self { value.0 }
}

impl From<&FileCov> for File {
	fn from(value: &FileCov) -> Self { value.0.clone() }
}

impl Hash for FileCov {
	fn hash<H: Hasher>(&self, state: &mut H) { UrlCov::from(&self.url).hash(state); }
}

impl PartialEq for FileCov {
	fn eq(&self, other: &Self) -> bool { UrlCov::from(&self.url).eq(&UrlCov::from(&other.url)) }
}

impl PartialEq<UrlBufCov> for FileCov {
	fn eq(&self, other: &UrlBufCov) -> bool { UrlCov::from(&self.url).eq(other) }
}

impl PartialEq<UrlBuf> for FileCov {
	fn eq(&self, other: &UrlBuf) -> bool { UrlCov::from(&self.url).eq(&UrlCov::from(other)) }
}

impl Eq for FileCov {}

impl Equivalent<FileCov> for UrlCov<'_> {
	fn equivalent(&self, key: &FileCov) -> bool { self.eq(&UrlCov::from(&key.url)) }
}

impl Serialize for FileCov {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		self.0.serialize(serializer)
	}
}

impl<'de> Deserialize<'de> for FileCov {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		File::deserialize(deserializer).map(Self)
	}
}

impl IntoLua for FileCov {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.0.into_lua(lua) }
}

impl FromLua for FileCov {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		File::from_lua(value, lua).map(Self)
	}
}
