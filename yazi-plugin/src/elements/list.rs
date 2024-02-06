use mlua::{AnyUserData, ExternalError, FromLua, Lua, Table, UserData, Value};
use ratatui::widgets::Widget;

use super::{Line, RectRef, Renderable, Span, Style};

// --- List
#[derive(Clone)]
pub struct List {
	area: ratatui::layout::Rect,

	inner: ratatui::widgets::List<'static>,
}

impl List {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		ui.set(
			"List",
			lua.create_function(|_, (area, items): (RectRef, Vec<ListItem>)| {
				Ok(Self { area: *area, inner: ratatui::widgets::List::new(items) })
			})?,
		)
	}
}

impl UserData for List {}

impl Renderable for List {
	fn area(&self) -> ratatui::layout::Rect { self.area }

	fn render(self: Box<Self>, buf: &mut ratatui::buffer::Buffer) {
		self.inner.render(self.area, buf);
	}

	fn clone_render(&self, buf: &mut ratatui::buffer::Buffer) { Box::new(self.clone()).render(buf) }
}

// --- ListItem
#[derive(Clone, FromLua)]
pub struct ListItem {
	content: ratatui::text::Text<'static>,
	style:   Option<ratatui::style::Style>,
}

impl ListItem {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		ui.set(
			"ListItem",
			lua.create_function(|_, value: Value| {
				match value {
					Value::UserData(ud) => {
						let content: ratatui::text::Text = if let Ok(line) = ud.take::<Line>() {
							line.0.into()
						} else if let Ok(span) = ud.take::<Span>() {
							span.0.into()
						} else {
							return Err("expected a String, Line or Span".into_lua_err());
						};
						return Ok(Self { content, style: None });
					}
					Value::String(s) => {
						return Ok(Self { content: s.to_string_lossy().into_owned().into(), style: None });
					}
					_ => {}
				}
				Err("expected a String, Line or Span".into_lua_err())
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

impl UserData for ListItem {
	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("style", |_, (ud, value): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.style = match value {
				Value::Nil => None,
				Value::Table(tb) => Some(Style::from(tb).0),
				Value::UserData(ud) => Some(ud.borrow::<Style>()?.0),
				_ => return Err("expected a Style or Table or nil".into_lua_err()),
			};
			Ok(ud)
		});
	}
}
