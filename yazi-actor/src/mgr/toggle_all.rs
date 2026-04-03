use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::mgr::ToggleAllForm;
use yazi_scheduler::NotifyProxy;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct ToggleAll;

impl Actor for ToggleAll {
	type Form = ToggleAllForm;

	const NAME: &str = "toggle_all";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		use either::Either::*;
		let tab = cx.tab_mut();

		let it = tab.current.files.iter().map(|f| &f.url);
		let either = match form.state {
			Some(true) if form.urls.is_empty() => Left((vec![], it.collect())),
			Some(true) => Right((vec![], form.urls)),
			Some(false) if form.urls.is_empty() => Left((it.collect(), vec![])),
			Some(false) => Right((form.urls, vec![])),
			None if form.urls.is_empty() => Left(it.partition(|&u| tab.selected.contains(u))),
			None => Right(form.urls.into_iter().partition(|u| tab.selected.contains(u))),
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
			NotifyProxy::push_warn(
				"Toggle all",
				"Some files cannot be selected, due to path nesting conflict.",
			);
		}
		succ!();
	}
}
