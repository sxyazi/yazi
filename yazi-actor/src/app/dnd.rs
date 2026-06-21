use anyhow::Result;
use mlua::{ObjectLike, Table};
use tracing::error;
use yazi_actor::lives::Lives;
use yazi_binding::runtime_scope;
use yazi_macro::succ;
use yazi_parser::app::DndForm;
use yazi_plugin::LUA;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Dnd;

impl Actor for Dnd {
	type Form = DndForm;

	const NAME: &str = "dnd";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some(size) = cx.term.as_ref().and_then(|t| t.size().ok()) else { succ!() };
		let area = yazi_binding::elements::Rect::from(size);

		let result = Lives::scope(cx.core, move |_| {
			runtime_scope!(LUA, "root", {
				let root = LUA.globals().raw_get::<Table>("Root")?.call_method::<Table>("new", area)?;

				if form.event.is_drag() {
					root.call_method::<()>("drag", form.event)?;
				} else {
					root.call_method::<()>("drop", form.event)?;
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
