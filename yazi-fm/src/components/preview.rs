use crossterm::event::MouseEventKind;
use mlua::{Table, TableExt};
use ratatui::{buffer::Buffer, widgets::Widget};
use yazi_plugin::{bindings::{Cast, MouseEvent}, LUA};

use crate::Ctx;

pub(crate) struct Preview<'a> {
	cx: &'a Ctx,
}

impl<'a> Preview<'a> {
	#[inline]
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }

	pub(crate) fn mouse(event: crossterm::event::MouseEvent) -> mlua::Result<()> {
		let evt = MouseEvent::cast(&LUA, event)?;
		let comp: Table = LUA.globals().raw_get("Preview")?;

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

impl Widget for Preview<'_> {
	fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
		let preview = &self.cx.manager.active().preview;
		let Some(lock) = &preview.lock else {
			return;
		};

		if (lock.window.rows, lock.window.cols) != (area.height, area.width) {
			return;
		}

		for w in &lock.data {
			w.clone_render(buf);
		}
	}
}
