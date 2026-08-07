use mlua::Lua;
use yazi_binding::{Runtime, Scope};

pub struct Runner {
	pub(super) setter: fn(&Lua) -> mlua::Result<()>,
}

impl Runner {
	pub fn spawn(&self, name: &str) -> mlua::Result<Lua> { self.spawn_with(Scope::default(), name) }

	pub fn spawn_with<S>(&self, scope: S, name: &str) -> mlua::Result<Lua>
	where
		S: Into<Scope>,
	{
		let lua = Lua::new();
		lua.set_app_data(Runtime::new(name, scope.into()));

		(self.setter)(&lua)?;
		Ok(lua)
	}
}
