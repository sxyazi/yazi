use mlua::{AnyUserData, Lua};

use crate::cast_to_renderable;

pub fn pour(lua: &Lua) -> mlua::Result<()> {
	let ui = lua.create_table()?;

	// Register
	super::Padding::register(lua)?;
	super::Rect::register(lua)?;

	// Install
	super::Bar::install(lua, &ui)?;
	super::Border::install(lua, &ui)?;
	super::Clear::install(lua, &ui)?;
	super::Constraint::install(lua, &ui)?;
	super::Gauge::install(lua, &ui)?;
	super::Layout::install(lua, &ui)?;
	super::Line::install(lua, &ui)?;
	super::List::install(lua, &ui)?;
	super::ListItem::install(lua, &ui)?;
	super::Padding::install(lua, &ui)?;
	super::Paragraph::install(lua, &ui)?;
	super::Position::install(lua, &ui)?;
	super::Rect::install(lua, &ui)?;
	super::Span::install(lua, &ui)?;
	super::Style::install(lua, &ui)?;

	lua.globals().raw_set("ui", ui)
}

pub trait Renderable {
	fn area(&self) -> ratatui::layout::Rect;

	fn render(self: Box<Self>, buf: &mut ratatui::buffer::Buffer);

	fn clone_render(&self, buf: &mut ratatui::buffer::Buffer);
}

pub fn render_widgets(widgets: Vec<AnyUserData>, buf: &mut ratatui::buffer::Buffer) {
	for widget in widgets {
		if let Some(w) = cast_to_renderable(widget) {
			w.render(buf);
		}
	}
}
