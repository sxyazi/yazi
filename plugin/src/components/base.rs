use mlua::{FromLua, Lua, Table, UserData, Value};
use ratatui::widgets::Widget;

use crate::{layout::Rect, GLOBALS, LUA};

#[derive(Clone)]
pub struct Base {
	area: ratatui::layout::Rect,

	kind: u8,
}

impl Base {
	pub(crate) fn install() -> mlua::Result<()> {
		let ui: Table = GLOBALS.get("ui")?;
		let base: Table = ui.get("Base")?;
		base.set(
			"new",
			LUA.create_function(|_, (area, kind): (Rect, u8)| Ok(Self { area: area.0, kind }))?,
		)
	}

	pub fn render(self, cx: &core::Ctx, buf: &mut ratatui::buffer::Buffer) {
		match self.kind {
			0 => super::Preview::new(cx).render(self.area, buf),
			_ => {}
		}
	}
}

impl<'lua> FromLua<'lua> for Base {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Base",
				message: Some("expected a Base".to_string()),
			}),
		}
	}
}

impl UserData for Base {}
