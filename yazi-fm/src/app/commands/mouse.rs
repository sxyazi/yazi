use anyhow::Result;
use crossterm::event::MouseEventKind;
use mlua::{ObjectLike, Table};
use tracing::error;
use yazi_actor::lives::Lives;
use yazi_config::YAZI;
use yazi_macro::succ;
use yazi_parser::app::MouseOpt;
use yazi_plugin::LUA;
use yazi_shared::data::Data;

use crate::app::App;

impl App {
	pub fn mouse(&mut self, opt: MouseOpt) -> Result<Data> {
		let event = yazi_plugin::bindings::MouseEvent::from(opt.event);
		let Some(size) = self.term.as_ref().and_then(|t| t.size().ok()) else { succ!() };

		let result = Lives::scope(&self.core, move || {
			let area = yazi_binding::elements::Rect::from(size);
			let root = LUA.globals().raw_get::<Table>("Root")?.call_method::<Table>("new", area)?;

			if matches!(event.kind, MouseEventKind::Down(_) if YAZI.mgr.mouse_events.get().draggable()) {
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

		if let Err(ref e) = result {
			error!("{e}");
		}
		succ!(result?);
	}
}
