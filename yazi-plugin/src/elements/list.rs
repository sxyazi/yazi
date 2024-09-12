use mlua::{ExternalError, FromLua, Lua, Table, UserData, Value};
use ratatui::widgets::Widget;

use super::{Line, RectRef, Renderable, Span};

// --- List
#[derive(Clone)]
pub struct List {
	area: ratatui::layout::Rect,

	inner: ratatui::widgets::List<'static>,
}

impl List {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		ui.raw_set(
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
#[derive(Clone, Default, FromLua)]
pub struct ListItem {
	content: ratatui::text::Text<'static>,
	style:   ratatui::style::Style,
}

impl ListItem {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		ui.raw_set(
			"ListItem",
			lua.create_function(|_, value: Value| match value {
				Value::String(s) => {
					Ok(Self { content: s.to_string_lossy().into_owned().into(), ..Default::default() })
				}
				Value::UserData(ud) => {
					let content: ratatui::text::Text = if let Ok(line) = ud.take::<Line>() {
						line.0.into()
					} else if let Ok(span) = ud.take::<Span>() {
						span.0.into()
					} else {
						return Err("expected a String, Line or Span".into_lua_err());
					};
					Ok(Self { content, ..Default::default() })
				}
				_ => Err("expected a String, Line or Span".into_lua_err()),
			})?,
		)
	}
}

impl From<ListItem> for ratatui::widgets::ListItem<'static> {
	fn from(value: ListItem) -> Self { Self::new(value.content).style(value.style) }
}

impl UserData for ListItem {
	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		crate::impl_style_method!(methods, style);
	}
}
