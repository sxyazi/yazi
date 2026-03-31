use mlua::Lua;

pub struct Runner {
	pub(super) setter: fn(&Lua) -> mlua::Result<()>,
}

impl Runner {
	pub fn spawn(&self, name: &str) -> mlua::Result<Lua> {
		let lua = Lua::new();
		lua.set_app_data(yazi_binding::Runtime::new_isolate(name));

		(self.setter)(&lua)?;
		Ok(lua)
	}
}
