use mlua::{IntoLua, Lua, Value};

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

impl IntoLua for Demand {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let t = lua.create_table_from([
			("append", self.append),
			("create", self.create),
			("create_new", self.create_new),
			("read", self.read),
			("truncate", self.truncate),
			("write", self.write),
		])?;
		t.into_lua(lua)
	}
}
