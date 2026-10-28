use anyhow::Result;
use yazi_config::YAZI;
use yazi_core::Ctx;
use yazi_macro::{relay, succ};
use yazi_parser::mgr::SeekForm;
use yazi_runner::previewer::SeekJob;
use yazi_shared::data::Data;

use crate::{Actor, act};

pub struct Seek;

impl Actor for Seek {
	type Form = SeekForm;

	const NAME: &str = "seek";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some(hovered) = cx.hovered() else {
			succ!(cx.preview_mut().reset());
		};

		let Some(mime) = cx.mgr.mimetype.get(&hovered.url) else {
			succ!(cx.preview_mut().reset());
		};

		let Some(previewer) = YAZI.plugin.previewers.matches(hovered, mime) else {
			succ!(cx.preview_mut().reset());
		};

		// Cancel the previous preview job if any
		cx.preview().scope.cancel();

		let job = SeekJob {
			file:  hovered.clone(),
			mime:  mime.to_owned().into(),
			units: form.units,
			scope: cx.scope.clone(),
		};

		let action = relay!(app:plugin, [previewer.name.clone()])
			.with_scope(&job)
			.with(1, job)
			.with("method", "seek");

		act!(app:plugin, cx.nested(&action)?, action)
	}
}
