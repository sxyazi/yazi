use crossterm::event::MouseEventKind;
use mlua::{Table, TableExt};
use yazi_plugin::{bindings::{Cast, MouseEvent}, LUA};

pub(crate) struct Parent;

impl Parent {
	pub(crate) fn mouse(event: crossterm::event::MouseEvent) -> mlua::Result<()> {
		let evt = MouseEvent::cast(&LUA, event)?;
		let comp: Table = LUA.globals().raw_get("Parent")?;

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
