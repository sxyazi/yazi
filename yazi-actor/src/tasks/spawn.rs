use anyhow::Result;
use yazi_core::tasks::TaskOpt;
use yazi_macro::succ;
use yazi_parser::tasks::SpawnForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Spawn;

impl Actor for Spawn {
	type Form = SpawnForm;

	const NAME: &str = "spawn";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		succ!(match form.opt {
			TaskOpt::Cut(r#in) => cx.tasks.scheduler.file_cut(r#in),
			TaskOpt::Copy(r#in) => cx.tasks.scheduler.file_copy(r#in),

			TaskOpt::Plugin(r#in) => cx.tasks.scheduler.plugin_entry(r#in),
		})
	}
}
