use mlua::{FromLuaMulti, Lua, MultiValue, Table};
use yazi_fs::{FsHash64, file::{FileRef, FileSig}};
use yazi_shim::fs::Error;

pub struct FetchStatus {
	pub hash:  u64,
	pub retry: bool,
	pub error: Option<Error>,
}

impl FromLuaMulti for FetchStatus {
	fn from_lua_multi(values: MultiValue, lua: &Lua) -> mlua::Result<Self> {
		let (file, result): (FileRef, Table) = FromLuaMulti::from_lua_multi(values, lua)?;

		Ok(Self {
			hash:  file.borrow(|f| Ok(FileSig(f).hash_u64()))?,
			retry: result.raw_get("retry")?,
			error: result.raw_get("error")?,
		})
	}
}
