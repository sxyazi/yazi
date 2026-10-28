use std::io;

use mlua::{BorrowedBytes, ExternalError, FromLuaMulti, Lua, MultiValue};

pub struct SeekFrom(pub io::SeekFrom);

impl FromLuaMulti for SeekFrom {
	fn from_lua_multi(values: MultiValue, lua: &Lua) -> mlua::Result<Self> {
		let (whence, offset): (Option<BorrowedBytes>, Option<i64>) =
			FromLuaMulti::from_lua_multi(values, lua)?;
		let offset = offset.unwrap_or_default();

		Ok(Self(match whence.as_deref().unwrap_or(b"cur") {
			b"set" if offset < 0 => {
				return Err("Seek offset must be non-negative for 'set'".into_lua_err());
			}
			b"set" => io::SeekFrom::Start(offset as u64),
			b"cur" => io::SeekFrom::Current(offset),
			b"end" => io::SeekFrom::End(offset),
			_ => return Err("Seek origin must be 'set', 'cur', or 'end'".into_lua_err()),
		}))
	}
}
