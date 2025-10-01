use anyhow::Result;
use yazi_config::popup::ConfirmCfg;
use yazi_macro::{act, succ};
use yazi_parser::mgr::RemoveOpt;
use yazi_proxy::{ConfirmProxy, MgrProxy};
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Remove;

impl Actor for Remove {
	type Options = RemoveOpt;

	const NAME: &str = "remove";

	fn act(cx: &mut Ctx, mut opt: Self::Options) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		opt.targets = if opt.hovered {
			cx.hovered().map_or(vec![], |h| vec![h.url.clone()])
		} else {
			cx.tab().selected_or_hovered().cloned().collect()
		};

		if opt.targets.is_empty() {
			succ!();
		} else if opt.force {
			return act!(mgr:remove_do, cx, opt);
		}

		let confirm = ConfirmProxy::show(if opt.permanently {
			ConfirmCfg::delete(&opt.targets)
		} else {
			ConfirmCfg::trash(&opt.targets)
		});

		tokio::spawn(async move {
			if confirm.await {
				MgrProxy::remove_do(opt.targets, opt.permanently);
			}
		});
		succ!();
	}
}

// --- Do
pub struct RemoveDo;

impl Actor for RemoveDo {
	type Options = RemoveOpt;

	const NAME: &str = "remove_do";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let mgr = &mut cx.mgr;

		mgr.tabs.iter_mut().for_each(|t| {
			t.selected.remove_many(&opt.targets);
		});

		for u in &opt.targets {
			mgr.yanked.remove(u);
		}

		mgr.yanked.catchup_revision(false);
		cx.tasks.file_remove(opt.targets, opt.permanently);
		succ!();
	}
}
