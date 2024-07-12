use crossterm::event::{MouseEvent, MouseEventKind};
use mlua::{Table, TableExt};
use tracing::error;
use yazi_config::MANAGER;
use yazi_plugin::{bindings::Cast, LUA};

use crate::{app::App, lives::Lives};

pub struct Opt {
	event: MouseEvent,
}

impl From<MouseEvent> for Opt {
	fn from(event: MouseEvent) -> Self { Self { event } }
}

impl App {
	pub(crate) fn mouse(&mut self, opt: impl Into<Opt>) {
		let event = (opt.into() as Opt).event;

		let Some(size) = self.term.as_ref().and_then(|t| t.size().ok()) else { return };
		let Ok(evt) = yazi_plugin::bindings::MouseEvent::cast(&LUA, event) else { return };

		let res = Lives::scope(&self.cx, move |_| {
			let area = yazi_plugin::elements::Rect::cast(&LUA, size)?;
			let root = LUA.globals().raw_get::<_, Table>("Root")?.call_method::<_, Table>("new", area)?;

			if matches!(event.kind, MouseEventKind::Down(_) if MANAGER.mouse_events.draggable()) {
				root.raw_set("_drag_start", evt.clone())?;
			}

			match event.kind {
				MouseEventKind::Down(_) => root.call_method("click", (evt, false))?,
				MouseEventKind::Up(_) => root.call_method("click", (evt, true))?,

				MouseEventKind::ScrollDown => root.call_method("scroll", (evt, 1))?,
				MouseEventKind::ScrollUp => root.call_method("scroll", (evt, -1))?,

				MouseEventKind::ScrollRight => root.call_method("touch", (evt, 1))?,
				MouseEventKind::ScrollLeft => root.call_method("touch", (evt, -1))?,

				MouseEventKind::Moved => root.call_method("move", evt)?,
				MouseEventKind::Drag(_) => root.call_method("drag", evt)?,
			}

			Ok(())
		});

		if let Err(e) = res {
			error!("{e}");
		}
	}
}
