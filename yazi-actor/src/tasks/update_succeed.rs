use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::tasks::UpdateSucceedForm;
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct UpdateSucceed;

impl Actor for UpdateSucceed {
	type Form = UpdateSucceedForm;

	const NAME: &str = "update_succeed";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		if form.urls.is_empty() {
			succ!();
		}

		if form.track
			&& form.id == cx.tasks.scheduler.behavior.first_id()
			&& let Some((parent, urn)) = form.urls[0].pair()
			&& parent == *cx.cwd()
		{
			cx.current_mut().trace = Some(urn.into());
			act!(mgr:hover, cx)?;
		}

		cx.mgr.watcher.report(form.urls);
		succ!();
	}
}
