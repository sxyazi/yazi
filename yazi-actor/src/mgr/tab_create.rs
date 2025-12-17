use anyhow::Result;
use yazi_core::tab::Tab;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::{CdSource, TabCreateOpt};
use yazi_proxy::AppProxy;
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

const MAX_TABS: usize = 9;

pub struct TabCreate;

impl Actor for TabCreate {
	type Options = TabCreateOpt;

	const NAME: &str = "tab_create";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		if cx.tabs().len() >= MAX_TABS {
			succ!(AppProxy::notify_warn(
				"Too many tabs",
				"You can only open up to 9 tabs at the same time."
			));
		}

		let mut tab = Tab::default();
		let (cd, url) = if let Some(wd) = opt.url {
			(true, wd.into_owned())
		} else if let Some(h) = cx.hovered() {
			tab.pref = cx.tab().pref.clone();
			(false, h.url.clone())
		} else if cx.cwd().is_search() {
			tab.pref = cx.tab().pref.clone();
			(true, cx.cwd().to_regular()?)
		} else {
			tab.pref = cx.tab().pref.clone();
			(true, cx.cwd().clone())
		};

		let tabs = &mut cx.mgr.tabs;
		tabs.items.insert(tabs.cursor + 1, tab);
		tabs.set_idx(tabs.cursor + 1);

		let cx = &mut Ctx::renew(cx);
		if cd {
			act!(mgr:cd, cx, (url, CdSource::Tab))?;
		} else {
			act!(mgr:reveal, cx, (url, CdSource::Tab))?;
		}

		act!(mgr:refresh, cx)?;
		act!(mgr:peek, cx, true)?;
		succ!(render!());
	}
}
