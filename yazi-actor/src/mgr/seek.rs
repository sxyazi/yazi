use anyhow::Result;
use mlua::ObjectLike;
use yazi_config::YAZI;
use yazi_core::{Ctx, app::PluginOpt};
use yazi_macro::succ;
use yazi_parser::mgr::SeekForm;
use yazi_runner::previewer::SeekJob;
use yazi_shared::{data::Data, pool::InternStr};

use crate::{Actor, act};

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

		let Some(previewer) = YAZI.plugin.previewers.matches(hovered, mime) else {
			succ!(cx.tab_mut().preview.reset());
		};

		let job = SeekJob {
			tab: cx.tab().id,
			previewer,
			file: hovered.clone(),
			mime: mime.intern(),
			units: form.units,
		};

		let opt = PluginOpt::new_callback(job, |_, plugin, job| plugin.call_method("seek", job));
		act!(app:plugin, cx, opt)
	}
}
