use hashbrown::HashMap;
use mlua::{IntoLua, Lua, Value};
use yazi_binding::elements::Rect;
use yazi_config::{LAYOUT, plugin::PreviewerArc};
use yazi_fs::file::File;
use yazi_shared::{data::Data, id::Id, pool::Symbol, scope::{Scope, ScopeLike}, sendable::Sendable};
use yazi_shim::SStr;

use crate::PluginJob;

#[derive(Clone, Debug)]
pub struct PeekJob {
	pub previewer: PreviewerArc,
	pub file:      File,
	pub mime:      Symbol<str>,
	pub sig:       Id,
	pub skip:      usize,
	pub scope:     Scope,
}

impl PluginJob for PeekJob {
	fn name(&self) -> &SStr { &self.previewer.name }
}

impl ScopeLike for PeekJob {
	fn scope(&self) -> &Scope { &self.scope }
}

impl IntoLua for PeekJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("scope", self.scope.into_lua(lua)?),
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
	pub file:  File,
	pub mime:  SStr,
	pub units: i16,
	pub scope: Scope,
}

impl From<SeekJob> for Data {
	fn from(value: SeekJob) -> Self {
		HashMap::from([
			("scope".into(), value.scope.into()),
			("area".into(), Rect::from(LAYOUT.get().preview).into()),
			("file".into(), value.file.into()),
			("mime".into(), value.mime.into()),
			("units".into(), value.units.into()),
		])
		.into()
	}
}

impl ScopeLike for SeekJob {
	fn scope(&self) -> &Scope { &self.scope }
}
