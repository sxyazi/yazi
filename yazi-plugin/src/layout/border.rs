use mlua::{AnyUserData, FromLua, Lua, Table, UserData, Value};
use ratatui::widgets::Widget;

use super::{Rect, Style};
use crate::{GLOBALS, LUA};

#[derive(Clone)]
pub struct Border {
	area: ratatui::layout::Rect,

	position: ratatui::widgets::Borders,
	type_:    ratatui::widgets::BorderType,
	style:    Option<ratatui::style::Style>,
}

impl Border {
	pub(crate) fn install() -> mlua::Result<()> {
		let ui: Table = GLOBALS.get("ui")?;
		let border: Table = ui.get("Border")?;
		border.set(
			"new",
			LUA.create_function(|_, (area, position): (Rect, u8)| {
				Ok(Self {
					area:     area.0,
					position: ratatui::widgets::Borders::from_bits_truncate(position),
					type_:    Default::default(),
					style:    Default::default(),
				})
			})?,
		)
	}

	pub fn render(self, buf: &mut ratatui::buffer::Buffer) {
		let mut block =
			ratatui::widgets::Block::default().borders(self.position).border_type(self.type_);

		if let Some(style) = self.style {
			block = block.border_style(style);
		}

		block.render(self.area, buf);
	}
}

impl<'lua> FromLua<'lua> for Border {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Border",
				message: Some("expected a Border".to_string()),
			}),
		}
	}
}

impl UserData for Border {
	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("type", |_, (ud, value): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.type_ = match value {
				1 => ratatui::widgets::BorderType::Rounded,
				2 => ratatui::widgets::BorderType::Double,
				3 => ratatui::widgets::BorderType::Thick,
				_ => ratatui::widgets::BorderType::Plain,
			};
			Ok(ud)
		});
		methods.add_function("style", |_, (ud, value): (AnyUserData, Value)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				match value {
					Value::Nil => me.style = None,
					Value::Table(tbl) => me.style = Some(Style::from(tbl).0),
					Value::UserData(ud) => me.style = Some(ud.borrow::<Style>()?.0),
					_ => return Err(mlua::Error::external("expected a Style or Table or nil")),
				}
			}
			Ok(ud)
		});
	}
}
