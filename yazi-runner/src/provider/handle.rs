use mlua::{FromLua, Lua, Table, Value};
use yazi_shared::{data::Data, id::Id};

#[derive(Clone)]
pub struct Handle {
	pub id:       Id,
	pub seekless: bool,
	pub state:    Data,
}

impl Handle {
	pub(super) fn into_table(self, t: &Table) -> mlua::Result<()> {
		t.raw_set("id", self.id)?;
		t.raw_set("seekless", self.seekless)?;
		t.raw_set("state", self.state)?;
		Ok(())
	}
}

impl FromLua for Handle {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;

		Ok(Self {
			id:       t.raw_get("id")?,
			seekless: t.raw_get("seekless")?,
			state:    t.raw_get("state")?,
		})
	}
}
