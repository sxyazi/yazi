use mlua::{FromLua, UserData};
use yazi_adapter::Dimension;

#[derive(Debug, Clone, Copy, FromLua)]
pub struct Window {
	pub rows:   u16,
	pub cols:   u16,
	pub width:  u16,
	pub height: u16,
}

impl Default for Window {
	fn default() -> Self {
		let ws = Dimension::available();
		Self { rows: ws.rows, cols: ws.columns, width: ws.width, height: ws.height }
	}
}

impl UserData for Window {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("rows", |_, me| Ok(me.rows));
		fields.add_field_method_get("cols", |_, me| Ok(me.cols));
		fields.add_field_method_get("width", |_, me| Ok(me.width));
		fields.add_field_method_get("height", |_, me| Ok(me.height));
	}
}
