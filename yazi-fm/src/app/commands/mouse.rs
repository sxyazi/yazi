use crossterm::event::{MouseEvent, MouseEventKind};
use mlua::Table;
use ratatui::layout::{Position, Rect};
use tracing::error;
use yazi_config::{LAYOUT, MANAGER};
use yazi_plugin::{bindings::Cast, LUA};

use crate::{app::App, components, lives::Lives};

pub struct Opt {
	event: MouseEvent,
}

impl From<MouseEvent> for Opt {
	fn from(event: MouseEvent) -> Self { Self { event } }
}

impl App {
	pub(crate) fn mouse(&mut self, opt: impl Into<Opt>) {
		let event = (opt.into() as Opt).event;

		let layout = LAYOUT.load();
		let position = Position { x: event.column, y: event.row };

		if matches!(event.kind, MouseEventKind::Moved | MouseEventKind::Drag(_)) {
			self.mouse_do(crate::Root::mouse, event, None);
			return;
		}

		if layout.current.contains(position) {
			self.mouse_do(components::Current::mouse, event, Some(layout.current));
		} else if layout.preview.contains(position) {
			self.mouse_do(components::Preview::mouse, event, Some(layout.preview));
		} else if layout.parent.contains(position) {
			self.mouse_do(components::Parent::mouse, event, Some(layout.parent));
		} else if layout.header.contains(position) {
			self.mouse_do(components::Header::mouse, event, Some(layout.header));
		} else if layout.status.contains(position) {
			self.mouse_do(components::Status::mouse, event, Some(layout.status));
		}
	}

	fn mouse_do(
		&self,
		f: impl FnOnce(MouseEvent) -> mlua::Result<()>,
		mut event: MouseEvent,
		rect: Option<Rect>,
	) {
		if matches!(event.kind, MouseEventKind::Down(_) if MANAGER.mouse_events.draggable()) {
			let evt = yazi_plugin::bindings::MouseEvent::cast(&LUA, event);
			if let (Ok(evt), Ok(root)) = (evt, LUA.globals().raw_get::<_, Table>("Root")) {
				root.raw_set("drag_start", evt).ok();
			}
		}

		if let Some(rect) = rect {
			event.row -= rect.y;
			event.column -= rect.x;
		}

		if let Err(e) = Lives::scope(&self.cx, move |_| f(event)) {
			error!("{:?}", e);
		}
	}
}
