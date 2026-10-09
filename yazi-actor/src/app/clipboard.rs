use anyhow::Result;
use mlua::{ObjectLike, Table};
use yazi_actor::lives::Lives;
use yazi_binding::runtime_scope;
use yazi_core::Ctx;
use yazi_macro::{log_if_err, succ};
use yazi_parser::app::ClipboardForm;
use yazi_plugin::LUA;
use yazi_shared::data::Data;

use crate::Actor;

pub struct Clipboard;

impl Actor for Clipboard {
	type Form = ClipboardForm;

	const NAME: &str = "clipboard";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some(size) = cx.term.as_ref().and_then(|t| t.size().ok()) else { succ!() };
		let area = yazi_binding::elements::Rect::from(size);

		let result = Lives::scope(cx, move |cx| {
			runtime_scope!(cx, "root", {
				let root = LUA.globals().raw_get::<Table>("Root")?.call_method::<Table>("new", area)?;
				root.call_method::<()>("clipboard", form.event)
			})
		});

		log_if_err!("Clipboard event handler", &result);
		succ!(result?);
	}
}
