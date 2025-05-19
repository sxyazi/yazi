use mlua::{FromLua, IntoLua, Lua, UserData, Value};

#[derive(Clone, Copy, Default, FromLua)]
pub struct Constraint(pub(super) ratatui::layout::Constraint);

impl Constraint {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		use ratatui::layout::Constraint as C;

		lua
			.create_table_from([
				("Min", lua.create_function(|_, n: u16| Ok(Self(C::Min(n))))?),
				("Max", lua.create_function(|_, n: u16| Ok(Self(C::Max(n))))?),
				("Length", lua.create_function(|_, n: u16| Ok(Self(C::Length(n))))?),
				("Percentage", lua.create_function(|_, n: u16| Ok(Self(C::Percentage(n))))?),
				("Ratio", lua.create_function(|_, (a, b): (u32, u32)| Ok(Self(C::Ratio(a, b))))?),
				("Fill", lua.create_function(|_, n: u16| Ok(Self(C::Fill(n))))?),
			])?
			.into_lua(lua)
	}
}

impl From<Constraint> for ratatui::layout::Constraint {
	fn from(value: Constraint) -> Self { value.0 }
}

impl UserData for Constraint {}
