use mlua::{IntoLua, Lua, Value};
use yazi_binding::{File, elements::Rect};
use yazi_config::LAYOUT;
use yazi_dds::Sendable;
use yazi_shared::{event::Action, pool::Symbol};

#[derive(Clone, Debug)]
pub struct PeekJob {
	pub action: &'static Action,
	pub file:   yazi_fs::File,
	pub mime:   Symbol<str>,
	pub skip:   usize,
}

impl IntoLua for PeekJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("area", Rect::from(LAYOUT.get().preview).into_lua(lua)?),
				("args", Sendable::args_to_table_ref(lua, &self.action.args)?.into_lua(lua)?),
				("file", File::new(self.file).into_lua(lua)?),
				("mime", self.mime.into_lua(lua)?),
				("skip", self.skip.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}

// --- Seek
#[derive(Clone, Debug)]
pub struct SeekJob {
	pub file:  yazi_fs::File,
	pub units: i16,
}

impl IntoLua for SeekJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("area", Rect::from(LAYOUT.get().preview).into_lua(lua)?),
				("file", File::new(self.file).into_lua(lua)?),
				("units", self.units.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
