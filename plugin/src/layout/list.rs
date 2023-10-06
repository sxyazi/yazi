use mlua::{AnyUserData, FromLua, Lua, Table, UserData, Value};
use ratatui::widgets::Widget;

use super::{Line, Rect, Span, Style};
use crate::{GLOBALS, LUA};

// --- List
#[derive(Clone)]
pub(crate) struct List {
	area: ratatui::layout::Rect,

	inner: ratatui::widgets::List<'static>,
}

impl List {
	pub(crate) fn install() -> mlua::Result<()> {
		let ui: Table = GLOBALS.get("ui")?;
		ui.set(
			"List",
			LUA.create_function(|_, (area, items): (Rect, Vec<ListItem>)| {
				let items: Vec<_> = items.into_iter().map(|x| x.into()).collect();
				Ok(Self { area: area.0, inner: ratatui::widgets::List::new(items) })
			})?,
		)
	}

	pub(crate) fn render(self, buf: &mut ratatui::buffer::Buffer) {
		self.inner.render(self.area, buf);
	}
}

impl<'lua> FromLua<'lua> for List {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "List",
				message: Some("expected a List".to_string()),
			}),
		}
	}
}

impl UserData for List {}

// --- ListItem
#[derive(Clone)]
pub(crate) struct ListItem {
	content: ratatui::text::Text<'static>,
	style:   Option<ratatui::style::Style>,
}

impl ListItem {
	pub(crate) fn install() -> mlua::Result<()> {
		let ui: Table = GLOBALS.get("ui")?;
		ui.set(
			"ListItem",
			LUA.create_function(|_, value: Value| {
				match value {
					Value::UserData(ud) => {
						let content: ratatui::text::Text = if let Ok(line) = ud.take::<Line>() {
							line.0.into()
						} else if let Ok(span) = ud.take::<Span>() {
							span.0.into()
						} else {
							return Err(mlua::Error::external("expected a String, Line or Span"));
						};
						return Ok(Self { content, style: None });
					}
					Value::String(s) => {
						return Ok(Self { content: s.to_str()?.to_string().into(), style: None });
					}
					_ => {}
				}
				Err(mlua::Error::external("expected a String, Line or Span"))
			})?,
		)
	}
}

impl From<ListItem> for ratatui::widgets::ListItem<'static> {
	fn from(value: ListItem) -> Self {
		let mut item = Self::new(value.content);
		if let Some(style) = value.style {
			item = item.style(style)
		}
		item
	}
}

impl<'lua> FromLua<'lua> for ListItem {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "ListItem",
				message: Some("expected a ListItem".to_string()),
			}),
		}
	}
}

impl UserData for ListItem {
	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("style", |_, (ud, value): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.style = match value {
				Value::Nil => None,
				Value::Table(tbl) => Some(Style::from(tbl).0),
				Value::UserData(ud) => Some(ud.borrow::<Style>()?.0),
				_ => return Err(mlua::Error::external("expected a Style or Table or nil")),
			};
			Ok(ud)
		});
	}
}
