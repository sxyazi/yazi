use anyhow::Result;
use mlua::{ObjectLike, Table};
use tracing::error;
use yazi_actor::lives::Lives;
use yazi_binding::runtime_scope;
use yazi_macro::succ;
use yazi_parser::app::ClipboardForm;
use yazi_plugin::LUA;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Clipboard;

impl Actor for Clipboard {
	type Form = ClipboardForm;

	const NAME: &str = "clipboard";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let event = yazi_binding::ClipboardEvent::from(form.event);

		let Some(size) = cx.term.as_ref().and_then(|t| t.size().ok()) else { succ!() };
		let area = yazi_binding::elements::Rect::from(size);

		let result = Lives::scope(cx.core, move |_| {
			runtime_scope!(LUA, "root", {
				let root = LUA.globals().raw_get::<Table>("Root")?.call_method::<Table>("new", area)?;

				root.call_method::<()>("clipboard", event)?;

				Ok(())
			})
		});

		if let Err(ref e) = result {
			error!("{e}");
		}
		succ!(result?);
	}
}
