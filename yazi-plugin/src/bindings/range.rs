use mlua::{IntoLua, Lua};

pub struct Range<T>(std::ops::Range<T>);

impl<T> From<std::ops::Range<T>> for Range<T> {
	fn from(value: std::ops::Range<T>) -> Self { Self(value) }
}

impl<T> IntoLua for Range<T>
where
	T: IntoLua,
{
	fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
		lua.create_sequence_from([self.0.start, self.0.end])?.into_lua(lua)
	}
}
