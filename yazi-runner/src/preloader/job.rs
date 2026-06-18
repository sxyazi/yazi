use mlua::{IntoLua, Lua, Value};
use yazi_binding::elements::Rect;
use yazi_config::{LAYOUT, plugin::PreloaderArc};
use yazi_fs::file::File;
use yazi_shared::data::Sendable;

pub struct PreloadJob {
	pub preloader: PreloaderArc,
	pub file:      File,
}

impl IntoLua for PreloadJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("area", Rect::from(LAYOUT.get().preview).into_lua(lua)?),
				("args", Sendable::args_to_table_ref(lua, &self.preloader.args)?.into_lua(lua)?),
				("file", self.file.into_lua(lua)?),
				("skip", 0.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
