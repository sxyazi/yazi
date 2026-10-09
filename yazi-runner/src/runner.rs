use mlua::Lua;
use yazi_binding::runtime::{Runtime, RuntimeSeed};

use crate::PluginJob;

pub struct Runner {
	pub(super) setter: fn(&Lua) -> mlua::Result<()>,
}

impl Runner {
	pub(crate) fn spawn(&self, job: &impl PluginJob) -> mlua::Result<Lua> {
		self.spawn_with(RuntimeSeed::new(job.tab(), job.name().as_ref(), Default::default()))
	}

	pub(crate) fn spawn_with(&self, seed: RuntimeSeed) -> mlua::Result<Lua> {
		let lua = Lua::new();
		lua.set_app_data(Runtime::new(seed));

		(self.setter)(&lua)?;
		Ok(lua)
	}
}
