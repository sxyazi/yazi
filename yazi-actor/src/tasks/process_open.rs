use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::tasks::ProcessOpenForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct ProcessOpen;

impl Actor for ProcessOpen {
	type Form = ProcessOpenForm;

	const NAME: &str = "process_open";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let done = cx.tasks.scheduler.process_open(form.opt);

		if let Some(replier) = form.replier {
			tokio::spawn(async move {
				replier.send(Ok(done.future().await.into())).ok();
			});
		}

		succ!();
	}
}
