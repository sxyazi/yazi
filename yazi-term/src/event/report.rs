use compact_str::CompactString;
use mlua::{ExternalError, IntoLua, Lua, Value};
use strum::EnumIs;

#[derive(Clone, Debug, EnumIs, Eq, PartialEq)]
pub enum Report {
	CsiU(u8),
	CursorBlink(bool),
	CursorShape(u8),
	Da1(Vec<u16>),
	XtVersion(CompactString),
	CellPixelSize { width: u16, height: u16 },
	BackgroundColor([u16; 3]),
	ColorScheme(bool),
	KittyGraphics { id: u32, ok: bool },
	Clipboard(bool),
}

impl IntoLua for Report {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
