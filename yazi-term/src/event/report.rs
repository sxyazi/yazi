use compact_str::CompactString;
use mlua::{ExternalError, IntoLua, Lua, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Report {
	CursorBlink(bool),
	CursorShape(u8),
	Da1(Vec<u16>),
	XtVersion(CompactString),
	CellPixelSize { width: u16, height: u16 },
	BackgroundColor([u16; 3]),
	KittyGraphics { id: u32, ok: bool },
}

impl IntoLua for Report {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
