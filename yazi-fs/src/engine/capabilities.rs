use mlua::{FromLua, Lua, Table, Value};

#[derive(Clone, Copy, Debug, Default)]
pub struct Capabilities {
	pub symlink:          bool,
	pub hard_link:        bool,
	pub trash:            bool,
	pub copy_progressive: bool,
}

impl FromLua for Capabilities {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;

		Ok(Self {
			symlink:          t.raw_get("symlink")?,
			hard_link:        t.raw_get("hard_link")?,
			trash:            t.raw_get("trash")?,
			copy_progressive: t.raw_get("copy_progressive")?,
		})
	}
}
