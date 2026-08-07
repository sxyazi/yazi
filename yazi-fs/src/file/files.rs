use std::ops::{Deref, DerefMut};

use mlua::{FromLua, IntoLua, Lua, Value};
use yazi_shared::url::UrlBuf;

use crate::{FsHash64, file::{File, FileSig}};

#[derive(Clone, Debug, Default)]
pub struct Files(pub Vec<File>);

impl Deref for Files {
	type Target = Vec<File>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Files {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
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

impl Files {
	pub fn hashes(&self) -> impl Iterator<Item = u64> + '_ {
		self.iter().map(|f| FileSig(f).hash_u64())
	}
}

impl FromLua for Files {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Vec::<File>::from_lua(value, lua).map(Self)
	}
}

impl IntoLua for Files {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_sequence_from(self.0)?.into_lua(lua)
	}
}
