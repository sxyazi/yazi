use anyhow::{Result, bail};
use yazi_macro::{act, render, render_and, succ};
use yazi_parser::{VoidOpt, mgr::EscapeOpt};
use yazi_proxy::AppProxy;
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Escape;

impl Actor for Escape {
	type Options = EscapeOpt;

	const NAME: &str = "escape";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		if opt.is_empty() {
			_ = act!(mgr:escape_find, cx)? != false
				|| act!(mgr:escape_visual, cx)? != false
				|| act!(mgr:escape_filter, cx)? != false
				|| act!(mgr:escape_select, cx)? != false
				|| act!(mgr:escape_search, cx)? != false;
			succ!();
		}

		if opt.contains(EscapeOpt::FIND) {
			act!(mgr:escape_find, cx)?;
		}
		if opt.contains(EscapeOpt::VISUAL) {
			act!(mgr:escape_visual, cx)?;
		}
		if opt.contains(EscapeOpt::FILTER) {
			act!(mgr:escape_filter, cx)?;
		}
		if opt.contains(EscapeOpt::SELECT) {
			act!(mgr:escape_select, cx)?;
		}
		if opt.contains(EscapeOpt::SEARCH) {
			act!(mgr:escape_search, cx)?;
		}
		succ!();
	}
}

// --- Find
pub struct EscapeFind;

impl Actor for EscapeFind {
	type Options = VoidOpt;

	const NAME: &str = "escape_find";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		succ!(render_and!(cx.tab_mut().finder.take().is_some()))
	}
}

// --- Visual
pub struct EscapeVisual;

impl Actor for EscapeVisual {
	type Options = VoidOpt;

	const NAME: &str = "escape_visual";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();

		let select = tab.mode.is_select();
		let Some((_, indices)) = tab.mode.take_visual() else { succ!(false) };

		render!();
		let urls: Vec<_> =
			indices.into_iter().filter_map(|i| tab.current.files.get(i)).map(|f| &f.url).collect();

		if !select {
			tab.selected.remove_many(urls);
		} else if urls.len() != tab.selected.add_many(urls) {
			AppProxy::notify_warn(
				"Escape visual mode",
				"Some files cannot be selected, due to path nesting conflict.",
			);
			bail!("Some files cannot be selected, due to path nesting conflict.");
		}

		succ!(true)
	}
}

// --- Filter
pub struct EscapeFilter;

impl Actor for EscapeFilter {
	type Options = VoidOpt;

	const NAME: &str = "escape_filter";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		if cx.current_mut().files.filter().is_none() {
			succ!(false);
		}

		act!(mgr:filter_do, cx)?;
		render!();
		succ!(true);
	}
}

// --- Select
pub struct EscapeSelect;

impl Actor for EscapeSelect {
	type Options = VoidOpt;

	const NAME: &str = "escape_select";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();
		if tab.selected.is_empty() {
			succ!(false);
		}

		tab.selected.clear();
		if tab.hovered().is_some_and(|h| h.is_dir()) {
			act!(mgr:peek, cx, true)?;
		}

		render!();
		succ!(true);
	}
}

// --- Search
pub struct EscapeSearch;

impl Actor for EscapeSearch {
	type Options = VoidOpt;

	const NAME: &str = "escape_search";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let b = cx.cwd().is_search();
		act!(mgr:search_stop, cx)?;
		succ!(render_and!(b));
	}
}
