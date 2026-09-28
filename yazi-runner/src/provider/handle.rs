use mlua::{FromLua, Lua, Table, Value};
use yazi_shared::{data::Data, id::Id, sendable::Sendable};

#[derive(Clone)]
pub struct Handle {
	pub id:       Id,
	pub offset:   u64,
	pub seekless: bool,
	pub state:    Data,
}

impl Handle {
	pub(super) fn into_table(self, lua: &Lua, t: &Table) -> mlua::Result<()> {
		t.raw_set("id", self.id)?;
		t.raw_set("offset", Id(self.offset))?;
		t.raw_set("seekless", self.seekless)?;
		t.raw_set("state", Sendable::data_to_value(lua, self.state)?)?;
		Ok(())
	}
}

impl FromLua for Handle {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;

		Ok(Self {
			id:       t.raw_get("id")?,
			offset:   t.raw_get::<Id>("offset")?.get(),
			seekless: t.raw_get("seekless")?,
			state:    Sendable::value_to_data(lua, t.raw_get("state")?)?,
		})
	}
}
