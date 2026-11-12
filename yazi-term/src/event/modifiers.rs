use bitflags::bitflags;
use mlua::{IntoLua, Lua, Value};
use serde::Serialize;
use yazi_shim::CompactString;

bitflags! {
	#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize)]
	pub struct Modifiers: u8 {
		const SHIFT   = 1;
		const ALT     = 2;
		const CTRL    = 4;
		const SUPER   = 8;
		const HYPER   = 16;
		const META    = 32;
	}
}

impl Modifiers {
	pub fn is_literal(self) -> bool { self.difference(Self::SHIFT).is_empty() }

	pub(crate) fn for_char(c: char) -> Self {
		if c.is_uppercase() { Self::SHIFT } else { Self::empty() }
	}
}

impl IntoLua for Modifiers {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let t = lua.create_table_with_capacity(0, self.bits().count_ones() as usize)?;
		for (name, _) in self.iter_names() {
			t.raw_set(CompactString::from(name).into_ascii_lowercase(), true)?;
		}

		t.into_lua(lua)
	}
}
