use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::mgr::RemoveDoForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct RemoveDo;

impl Actor for RemoveDo {
	type Form = RemoveDoForm;

	const NAME: &str = "remove_do";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let mgr = &mut cx.mgr;

		mgr.tabs.iter_mut().for_each(|t| {
			t.selected.remove_many(&form.targets);
		});

		mgr.yanked.remove_many(&form.targets);
		mgr.yanked.catchup_revision(false);

		cx.tasks.file_remove(form.targets, form.permanently);
		succ!();
	}
}
