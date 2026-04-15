use std::sync::Arc;

use mlua::{IntoLua, Lua, Value};
use yazi_binding::{File, elements::Rect};
use yazi_config::LAYOUT;
use yazi_dds::Sendable;

pub struct PreloadJob {
	pub preloader: Arc<yazi_config::plugin::Preloader>,
	pub file:      yazi_fs::File,
}

impl IntoLua for PreloadJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("area", Rect::from(LAYOUT.get().preview).into_lua(lua)?),
				("args", Sendable::args_to_table_ref(lua, &self.preloader.args)?.into_lua(lua)?),
				("file", File::new(self.file).into_lua(lua)?),
				("skip", 0.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
