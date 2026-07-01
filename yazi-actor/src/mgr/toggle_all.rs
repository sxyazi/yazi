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

		let it = tab.current.entries.iter();
		let either = match form.state {
			Some(true) if form.files.is_empty() => Left((vec![], it.collect())),
			Some(true) => Right((vec![], form.files)),
			Some(false) if form.files.is_empty() => Left((it.collect(), vec![])),
			Some(false) => Right((form.files, vec![])),
			None if form.files.is_empty() => Left(it.partition(|&f| tab.selected.contains(f))),
			None => Right(form.files.into_iter().partition(|f| tab.selected.contains(f))),
		};

		let warn = match either {
			Left((removal, addition)) => {
				render!(tab.selected.remove_many(removal) > 0);
				addition.len() != render!(tab.selected.add_many(addition), > 0)
			}
			Right((removal, addition)) => {
				render!(tab.selected.remove_many(&removal) > 0);
				addition.len() != render!(tab.selected.add_many(&addition), > 0)
			}
		};

		if warn {
			NotifyProxy::push_warn(
				"Toggle all",
				"Some files cannot be selected due to path nesting conflict.",
			);
		}
		succ!();
	}
}
