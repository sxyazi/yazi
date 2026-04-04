use mlua::{IntoLua, Lua, Value};
use yazi_binding::{File, elements::Rect};
use yazi_config::LAYOUT;
use yazi_dds::Sendable;
use yazi_shared::event::Action;

pub struct PreloadJob {
	pub action: &'static Action,
	pub file:   yazi_fs::File,
}

impl IntoLua for PreloadJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("area", Rect::from(LAYOUT.get().preview).into_lua(lua)?),
				("args", Sendable::args_to_table_ref(lua, &self.action.args)?.into_lua(lua)?),
				("file", File::new(self.file).into_lua(lua)?),
				("skip", 0.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
