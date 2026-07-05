use anyhow::Result;
use yazi_config::popup::ConfirmCfg;
use yazi_fs::file::Files;
use yazi_macro::{act, confirm, succ};
use yazi_parser::mgr::{RemoveDoForm, RemoveForm};
use yazi_proxy::MgrProxy;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Remove;

impl Actor for Remove {
	type Form = RemoveForm;

	const NAME: &str = "remove";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		let targets = Files(if form.hovered {
			cx.hovered().map_or(vec![], |h| vec![h.clone()])
		} else {
			cx.tab().selected_or_hovered_files().cloned().collect()
		});

		if targets.is_empty() {
			succ!();
		} else if form.force {
			return act!(mgr:remove_do, cx, RemoveDoForm { permanently: form.permanently, targets: targets.into() });
		}

		let confirm = confirm!(
			cx,
			if form.permanently { ConfirmCfg::delete(&targets) } else { ConfirmCfg::trash(&targets) }
		)?;

		tokio::spawn(async move {
			if confirm.future().await {
				MgrProxy::remove_do(form.permanently, targets);
			}
		});
		succ!();
	}
}
