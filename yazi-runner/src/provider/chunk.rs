use std::io::{self, ErrorKind};

use mlua::{IntoLua, Lua, Value};
use yazi_shared::id::Id;

pub struct ProvideChunk {
	pub from:  u64,
	pub to:    u64,
	pub bytes: Vec<u8>,
}

impl ProvideChunk {
	pub fn new(from: u64, bytes: Vec<u8>) -> io::Result<Self> {
		let to = Self::to(from, bytes.len())?;
		Ok(Self { from, to, bytes })
	}

	pub fn to(from: u64, len: usize) -> io::Result<u64> {
		u64::try_from(len)
			.ok()
			.and_then(|len| from.checked_add(len))
			.ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "offset overflow"))
	}
}

impl IntoLua for ProvideChunk {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let t = lua.create_table_from([
			("from", Id(self.from).into_lua(lua)?),
			("to", Id(self.to).into_lua(lua)?),
			("bytes", lua.create_external_string(self.bytes)?.into_lua(lua)?),
		])?;

		t.into_lua(lua)
	}
}
