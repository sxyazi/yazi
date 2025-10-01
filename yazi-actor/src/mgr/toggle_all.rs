use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::mgr::ToggleAllOpt;
use yazi_proxy::AppProxy;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct ToggleAll;

impl Actor for ToggleAll {
	type Options = ToggleAllOpt;

	const NAME: &str = "toggle_all";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		use yazi_shared::Either::*;
		let tab = cx.tab_mut();

		let it = tab.current.files.iter().map(|f| &f.url);
		let either = match opt.state {
			Some(true) if opt.urls.is_empty() => Left((vec![], it.collect())),
			Some(true) => Right((vec![], opt.urls)),
			Some(false) if opt.urls.is_empty() => Left((it.collect(), vec![])),
			Some(false) => Right((opt.urls, vec![])),
			None if opt.urls.is_empty() => Left(it.partition(|&u| tab.selected.contains(u))),
			None => Right(opt.urls.into_iter().partition(|u| tab.selected.contains(u))),
		};

		let warn = match either {
			Left((removal, addition)) => {
				render!(tab.selected.remove_many(removal) > 0);
				addition.len() != render!(tab.selected.add_many(addition), > 0)
			}
			Right((removal, addition)) => {
				render!(tab.selected.remove_many(&removal) > 0);
				render!(tab.selected.add_many(&addition), > 0) != addition.len()
			}
		};

		if warn {
			AppProxy::notify_warn(
				"Toggle all",
				"Some files cannot be selected, due to path nesting conflict.",
			);
		}
		succ!();
	}
}
