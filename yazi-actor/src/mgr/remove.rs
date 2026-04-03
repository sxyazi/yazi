use anyhow::Result;
use yazi_config::popup::ConfirmCfg;
use yazi_macro::{act, succ};
use yazi_parser::mgr::RemoveForm;
use yazi_proxy::{ConfirmProxy, MgrProxy};
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Remove;

impl Actor for Remove {
	type Form = RemoveForm;

	const NAME: &str = "remove";

	fn act(cx: &mut Ctx, mut form: Self::Form) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		form.targets = if form.hovered {
			cx.hovered().map_or(vec![], |h| vec![h.url.clone()])
		} else {
			cx.tab().selected_or_hovered().cloned().collect()
		};

		if form.targets.is_empty() {
			succ!();
		} else if form.force {
			return act!(mgr:remove_do, cx, form);
		}

		let confirm = ConfirmProxy::show(if form.permanently {
			ConfirmCfg::delete(&form.targets)
		} else {
			ConfirmCfg::trash(&form.targets)
		});

		tokio::spawn(async move {
			if confirm.await {
				MgrProxy::remove_do(form.targets, form.permanently);
			}
		});
		succ!();
	}
}

// --- Do
pub struct RemoveDo;

impl Actor for RemoveDo {
	type Form = RemoveForm;

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
