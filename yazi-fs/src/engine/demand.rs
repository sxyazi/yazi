use mlua::{FromLua, IntoLua, Lua, Table, Value};

use super::{Attrs, FileBuilder};

#[derive(Clone, Copy, Default)]
pub struct Demand {
	pub attrs:      Attrs,
	pub append:     bool,
	pub create:     bool,
	pub create_new: bool,
	pub read:       bool,
	pub truncate:   bool,
	pub write:      bool,
}

impl TryFrom<Table> for Demand {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		Ok(Self {
			attrs:      t.raw_get("attrs")?,
			append:     t.raw_get("append")?,
			create:     t.raw_get("create")?,
			create_new: t.raw_get("create_new")?,
			read:       t.raw_get("read")?,
			truncate:   t.raw_get("truncate")?,
			write:      t.raw_get("write")?,
		})
	}
}

impl Demand {
	pub fn build<T: FileBuilder>(self) -> T {
		let mut demand = T::default();
		demand.attrs(self.attrs);

		if self.append {
			demand.append(true);
		}
		if self.create {
			demand.create(true);
		}
		if self.create_new {
			demand.create_new(true);
		}
		if self.read {
			demand.read(true);
		}
		if self.truncate {
			demand.truncate(true);
		}
		if self.write {
			demand.write(true);
		}
		demand
	}
}

impl FromLua for Demand {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Nil => Ok(Self::default()),
			_ => Table::from_lua(value, lua)?.try_into(),
		}
	}
}

impl IntoLua for Demand {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("attrs", self.attrs.into_lua(lua)?),
				("append", self.append.into_lua(lua)?),
				("create", self.create.into_lua(lua)?),
				("create_new", self.create_new.into_lua(lua)?),
				("read", self.read.into_lua(lua)?),
				("truncate", self.truncate.into_lua(lua)?),
				("write", self.write.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
