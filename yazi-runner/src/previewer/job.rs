use mlua::{IntoLua, Lua, Value};
use yazi_binding::elements::Rect;
use yazi_config::{LAYOUT, plugin::PreviewerArc};
use yazi_fs::file::File;
use yazi_shared::{id::Id, pool::Symbol, sendable::Sendable};
use yazi_shim::SStr;

use crate::PluginJob;

#[derive(Clone, Debug)]
pub struct PeekJob {
	pub tab:       Id,
	pub previewer: PreviewerArc,
	pub file:      File,
	pub mime:      Symbol<str>,
	pub sig:       Id,
	pub skip:      usize,
}

impl PluginJob for PeekJob {
	fn tab(&self) -> Id { self.tab }

	fn name(&self) -> &SStr { &self.previewer.name }
}

impl IntoLua for PeekJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tab", self.tab.into_lua(lua)?),
				("area", Rect::from(LAYOUT.get().preview).into_lua(lua)?),
				("args", Sendable::args_to_table_ref(lua, &self.previewer.args)?.into_lua(lua)?),
				("file", self.file.into_lua(lua)?),
				("mime", self.mime.into_lua(lua)?),
				("sig", self.sig.into_lua(lua)?),
				("skip", self.skip.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}

// --- Seek
#[derive(Clone, Debug)]
pub struct SeekJob {
	pub tab:       Id,
	pub previewer: PreviewerArc,
	pub file:      File,
	pub mime:      Symbol<str>,
	pub units:     i16,
}

impl PluginJob for SeekJob {
	fn tab(&self) -> Id { self.tab }

	fn name(&self) -> &SStr { &self.previewer.name }
}

impl IntoLua for SeekJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tab", self.tab.into_lua(lua)?),
				("area", Rect::from(LAYOUT.get().preview).into_lua(lua)?),
				("file", self.file.into_lua(lua)?),
				("mime", self.mime.into_lua(lua)?),
				("units", self.units.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
