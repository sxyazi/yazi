use crossterm::event::MouseEventKind;
use mlua::{Table, TableExt};
use ratatui::{buffer::Buffer, widgets::Widget};
use tracing::error;
use yazi_plugin::{bindings::{Cast, MouseEvent}, elements::{render_widgets, Rect}, LUA};

pub(crate) struct Header;

impl Widget for Header {
	fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
		let mut f = || {
			let area = Rect::cast(&LUA, area)?;
			let comp: Table = LUA.globals().raw_get("Header")?;
			render_widgets(comp.call_method("render", area)?, buf);
			Ok::<_, anyhow::Error>(())
		};
		if let Err(e) = f() {
			error!("{:?}", e);
		}
	}
}

impl Header {
	pub(crate) fn mouse(event: crossterm::event::MouseEvent) -> mlua::Result<()> {
		let evt = MouseEvent::cast(&LUA, event)?;
		let comp: Table = LUA.globals().raw_get("Header")?;

		match event.kind {
			MouseEventKind::Down(_) => comp.call_method("click", (evt, false))?,
			MouseEventKind::Up(_) => comp.call_method("click", (evt, true))?,
			MouseEventKind::ScrollDown => comp.call_method("scroll", (evt, 1))?,
			MouseEventKind::ScrollUp => comp.call_method("scroll", (evt, -1))?,
			MouseEventKind::ScrollRight => comp.call_method("touch", (evt, 1))?,
			MouseEventKind::ScrollLeft => comp.call_method("touch", (evt, -1))?,
			_ => (),
		}
		Ok(())
	}
}
