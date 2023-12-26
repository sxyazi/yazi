pub trait Cast<T> {
	fn cast(lua: &mlua::Lua, data: T) -> mlua::Result<mlua::AnyUserData>;
}
