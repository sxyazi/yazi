use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, Value};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "[u16; 3]")]
pub struct MgrRatio {
	parent:  u16,
	current: u16,
	preview: u16,
}

impl TryFrom<[u16; 3]> for MgrRatio {
	type Error = anyhow::Error;

	fn try_from(ratio: [u16; 3]) -> Result<Self, Self::Error> {
		if ratio.iter().all(|&r| r == 0) {
			bail!("at least one layout ratio must be non-zero: {:?}", ratio);
		}

		Ok(Self { parent: ratio[0], current: ratio[1], preview: ratio[2] })
	}
}

impl TryFrom<Table> for MgrRatio {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		Ok([t.raw_get(1)?, t.raw_get(2)?, t.raw_get(3)?].try_into()?)
	}
}

impl FromLua for MgrRatio {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => Err("expected a table".into_lua_err()),
		}
	}
}

impl IntoLua for MgrRatio {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_sequence_from([self.parent, self.current, self.preview])?.into_lua(lua)
	}
}
