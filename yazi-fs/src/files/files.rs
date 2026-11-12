use std::{ops::{Deref, DerefMut}, vec};

use mlua::{FromLua, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::url::UrlBuf;

use crate::{FsHash64, file::{File, FileSig}};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Files(pub Vec<File>);

impl Deref for Files {
	type Target = Vec<File>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Files {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<File> for Files {
	fn from(value: File) -> Self { Self(vec![value]) }
}

impl From<Vec<File>> for Files {
	fn from(value: Vec<File>) -> Self { Self(value) }
}

impl From<Files> for Vec<File> {
	fn from(value: Files) -> Self { value.0 }
}

impl From<Files> for Vec<UrlBuf> {
	fn from(value: Files) -> Self { value.0.into_iter().map(|f| f.url).collect() }
}

impl FromIterator<File> for Files {
	fn from_iter<T: IntoIterator<Item = File>>(iter: T) -> Self { Self(iter.into_iter().collect()) }
}

impl IntoIterator for Files {
	type IntoIter = vec::IntoIter<File>;
	type Item = File;

	fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'a> IntoIterator for &'a Files {
	type IntoIter = std::slice::Iter<'a, File>;
	type Item = &'a File;

	fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl Files {
	pub fn with_capacity(capacity: usize) -> Self { Self(Vec::with_capacity(capacity)) }

	pub fn hashes(&self) -> impl Iterator<Item = u64> + '_ {
		self.iter().map(|f| FileSig(f).hash_u64())
	}
}

impl Extend<File> for Files {
	fn extend<T: IntoIterator<Item = File>>(&mut self, iter: T) { self.0.extend(iter); }
}

impl FromLua for Files {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Vec::<File>::from_lua(value, lua).map(Self)
	}
}

impl IntoLua for Files {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.0.into_lua(lua) }
}
