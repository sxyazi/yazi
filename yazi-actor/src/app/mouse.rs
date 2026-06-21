use anyhow::Result;
use mlua::{ObjectLike, Table};
use tracing::error;
use yazi_actor::lives::Lives;
use yazi_binding::runtime_scope;
use yazi_macro::succ;
use yazi_parser::app::MouseForm;
use yazi_plugin::LUA;
use yazi_shared::data::Data;
use yazi_term::event::MouseEventKind;

use crate::{Actor, Ctx};

pub struct Mouse;

impl Actor for Mouse {
	type Form = MouseForm;

	const NAME: &str = "mouse";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some(size) = cx.term.as_ref().and_then(|t| t.size().ok()) else { succ!() };
		let area = yazi_binding::elements::Rect::from(size);

		let result = Lives::scope(cx.core, move |_| {
			runtime_scope!(LUA, "root", {
				let root = LUA.globals().raw_get::<Table>("Root")?.call_method::<Table>("new", area)?;

				match form.event.kind {
					MouseEventKind::Down(_) => root.call_method("click", (form.event, false))?,
					MouseEventKind::Up(_) => root.call_method("click", (form.event, true))?,

					MouseEventKind::ScrollDown => root.call_method("scroll", (form.event, 1))?,
					MouseEventKind::ScrollUp => root.call_method("scroll", (form.event, -1))?,

					MouseEventKind::ScrollRight => root.call_method("touch", (form.event, 1))?,
					MouseEventKind::ScrollLeft => root.call_method("touch", (form.event, -1))?,

					MouseEventKind::Moved => root.call_method("move", form.event)?,
					MouseEventKind::Drag(_) => root.call_method("drag", form.event)?,
				}

				Ok(())
			})
		});

		if let Err(ref e) = result {
			error!("{e}");
		}
		succ!(result?);
	}
}
