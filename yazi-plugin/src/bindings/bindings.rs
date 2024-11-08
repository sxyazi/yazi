use mlua::Lua;

pub trait Cast<T> {
	fn cast(lua: &Lua, data: T) -> mlua::Result<mlua::AnyUserData>;
}
