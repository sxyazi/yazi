use anyhow::Result;
use yazi_core::tab::Tab;
use yazi_macro::{act, render, succ};
use yazi_parser::{mgr::TabCreateOpt, tab::CdSource};
use yazi_proxy::AppProxy;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

const MAX_TABS: usize = 9;

pub struct TabCreate;

impl Actor for TabCreate {
	type Options = TabCreateOpt;

	const NAME: &'static str = "tab_create";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		if cx.tabs().len() >= MAX_TABS {
			succ!(AppProxy::notify_warn(
				"Too many tabs",
				"You can only open up to 9 tabs at the same time."
			));
		}

		let mut tab = Tab::default();
		let (cd, url) = if let Some(wd) = opt.wd {
			(true, wd)
		} else if let Some(h) = cx.hovered() {
			tab.pref = cx.tab().pref.clone();
			tab.apply_files_attrs();
			(false, h.url.to_regular())
		} else {
			tab.pref = cx.tab().pref.clone();
			tab.apply_files_attrs();
			(true, cx.cwd().to_regular())
		};

		let tabs = &mut cx.mgr.tabs;
		tabs.items.insert(tabs.cursor + 1, tab);
		tabs.set_idx(tabs.cursor + 1);

		let cx = &mut Ctx::active(cx.core);
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
