use mlua::IntoLua;

pub struct Range<T>(std::ops::Range<T>);

impl<T> From<std::ops::Range<T>> for Range<T> {
	fn from(value: std::ops::Range<T>) -> Self { Self(value) }
}

impl<'lua, T> IntoLua<'lua> for Range<T>
where
	T: IntoLua<'lua>,
{
	fn into_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
		lua.create_sequence_from([self.0.start, self.0.end])?.into_lua(lua)
	}
}
