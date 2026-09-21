use anyhow::Result;
use yazi_fs::CWD;
use yazi_macro::{succ, tab};
use yazi_parser::VoidForm;
use yazi_shared::{data::Data, url::UrlLike};
use yazi_watcher::MgrProxy;

use crate::{Actor, Ctx, act};

pub struct Refresh;

impl Actor for Refresh {
	type Form = VoidForm;

	const NAME: &str = "refresh";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		CWD.set(cx.active().cwd(), Self::cwd_changed);

		let tab = tab!(cx);
		cx.core.mgr.watcher.refresher.request(
			[Some(&mut tab.current), tab.parent.as_mut()]
				.into_iter()
				.flatten()
				.filter(|f| f.is_absolute() || !f.auth().is_local())
				.map(|f| f.take_refresh()),
		);

		act!(mgr:peek, cx)?;
		act!(mgr:watch, cx).ok();
		act!(mgr:update_paged, cx)?;

		cx.tasks.prework_sorted(&cx.current().entries);
		succ!();
	}
}

impl Refresh {
	fn cwd_changed() {
		if !CWD.load().auth().is_local() {
			MgrProxy::watch();
		}
	}
}
