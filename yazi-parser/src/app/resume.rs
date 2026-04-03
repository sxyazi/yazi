use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, Value};
use ratatui::layout::Rect;
use tokio::sync::mpsc;
use yazi_shared::event::Replier;

#[derive(Debug)]
pub struct ResumeForm {
	pub tx:      mpsc::UnboundedSender<(bool, Replier)>,
	pub reflow:  fn(Rect) -> mlua::Result<Table>,
	pub replier: Replier,
}

impl FromLua for ResumeForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ResumeForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
