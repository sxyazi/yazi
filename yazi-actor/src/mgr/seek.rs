use anyhow::Result;
use mlua::ObjectLike;
use yazi_config::YAZI;
use yazi_core::app::PluginOpt;
use yazi_macro::{act, succ};
use yazi_parser::mgr::SeekForm;
use yazi_runner::previewer::SeekJob;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Seek;

impl Actor for Seek {
	type Form = SeekForm;

	const NAME: &str = "seek";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some(hovered) = cx.hovered() else {
			succ!(cx.tab_mut().preview.reset());
		};

		let Some(mime) = cx.mgr.mimetype.get(&hovered.url) else {
			succ!(cx.tab_mut().preview.reset());
		};

		let Some(previewer) = YAZI.plugin.previewer(hovered, mime) else {
			succ!(cx.tab_mut().preview.reset());
		};

		let job = SeekJob { file: hovered.clone(), units: form.units };
		let opt = PluginOpt::new_callback(&*previewer.run.name, move |_, plugin| {
			plugin.call_method("seek", job)
		});
		act!(app:plugin, cx, opt)
	}
}
