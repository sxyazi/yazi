use mlua::{IntoLua, Lua, Value};
use yazi_binding::elements::Rect;
use yazi_config::{LAYOUT, plugin::PreloaderArc};
use yazi_fs::file::File;
use yazi_shared::{id::Id, pool::Symbol, sendable::Sendable};
use yazi_shim::SStr;

use crate::PluginJob;

pub struct PreloadJob {
	pub tab:       Id,
	pub preloader: PreloaderArc,
	pub file:      File,
	pub mime:      Symbol<str>,
}

impl PluginJob for PreloadJob {
	fn tab(&self) -> Id { self.tab }

	fn name(&self) -> &SStr { &self.preloader.name }
}

impl IntoLua for PreloadJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tab", self.tab.into_lua(lua)?),
				("area", Rect::from(LAYOUT.get().preview).into_lua(lua)?),
				("args", Sendable::args_to_table_ref(lua, &self.preloader.args)?.into_lua(lua)?),
				("file", self.file.into_lua(lua)?),
				("mime", self.mime.into_lua(lua)?),
				("skip", 0.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
