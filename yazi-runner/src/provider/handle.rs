use mlua::{FromLua, Lua, Table, Value};
use yazi_fs::engine::Random;
use yazi_shared::{data::Data, id::Id};

#[derive(Clone)]
pub struct Handle {
	pub id:     Id,
	pub random: Random,
	pub state:  Data,
}

impl Handle {
	pub(super) fn into_table(self, t: &Table) -> mlua::Result<()> {
		t.raw_set("id", self.id)?;
		t.raw_set("random", self.random)?;
		t.raw_set("state", self.state)?;
		Ok(())
	}
}

impl FromLua for Handle {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;

		Ok(Self { id: t.raw_get("id")?, random: t.raw_get("random")?, state: t.raw_get("state")? })
	}
}
