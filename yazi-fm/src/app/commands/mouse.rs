use crossterm::event::{MouseEvent, MouseEventKind};
use mlua::{ObjectLike, Table};
use tracing::error;
use yazi_config::YAZI;
use yazi_plugin::LUA;

use crate::{app::App, lives::Lives};

struct Opt {
	event: MouseEvent,
}

impl From<MouseEvent> for Opt {
	fn from(event: MouseEvent) -> Self { Self { event } }
}

impl App {
	#[yazi_codegen::command]
	pub fn mouse(&mut self, opt: Opt) {
		let event = yazi_plugin::bindings::MouseEvent::from(opt.event);
		let Some(size) = self.term.as_ref().and_then(|t| t.size().ok()) else { return };

		let res = Lives::scope(&self.cx, move || {
			let area = yazi_plugin::elements::Rect::from(size);
			let root = LUA.globals().raw_get::<Table>("Root")?.call_method::<Table>("new", area)?;

			if matches!(event.kind, MouseEventKind::Down(_) if YAZI.mgr.mouse_events.draggable()) {
				root.raw_set("_drag_start", event)?;
			}

			match event.kind {
				MouseEventKind::Down(_) => root.call_method("click", (event, false))?,
				MouseEventKind::Up(_) => root.call_method("click", (event, true))?,

				MouseEventKind::ScrollDown => root.call_method("scroll", (event, 1))?,
				MouseEventKind::ScrollUp => root.call_method("scroll", (event, -1))?,

				MouseEventKind::ScrollRight => root.call_method("touch", (event, 1))?,
				MouseEventKind::ScrollLeft => root.call_method("touch", (event, -1))?,

				MouseEventKind::Moved => root.call_method("move", event)?,
				MouseEventKind::Drag(_) => root.call_method("drag", event)?,
			}

			Ok(())
		});

		if let Err(e) = res {
			error!("{e}");
		}
	}
}
