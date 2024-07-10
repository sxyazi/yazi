use crossterm::event::MouseEventKind;
use mlua::{Table, TableExt};
use ratatui::widgets::Widget;
use tracing::error;
use yazi_plugin::{bindings::{Cast, MouseEvent}, elements::{render_widgets, Rect}, LUA};

pub(crate) struct Status;

impl Widget for Status {
	fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
		let mut f = || {
			let area = Rect::cast(&LUA, area)?;
			let comp: Table = LUA.globals().raw_get("Status")?;
			render_widgets(comp.call_method("render", area)?, buf);
			Ok::<_, anyhow::Error>(())
		};
		if let Err(e) = f() {
			error!("{:?}", e);
		}
	}
}

impl Status {
	pub(crate) fn mouse(event: crossterm::event::MouseEvent) -> mlua::Result<()> {
		let evt = MouseEvent::cast(&LUA, event)?;
		let comp: Table = LUA.globals().raw_get("Status")?;

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
