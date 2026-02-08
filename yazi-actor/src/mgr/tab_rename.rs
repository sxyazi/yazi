use std::borrow::Cow;

use anyhow::Result;
use yazi_config::popup::InputCfg;
use yazi_macro::{render, succ};
use yazi_parser::mgr::TabRenameOpt;
use yazi_proxy::{InputProxy, MgrProxy};
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct TabRename;

impl Actor for TabRename {
	type Options = TabRenameOpt;

	const NAME: &str = "tab_rename";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tab = cx.active().id;
		let pref = &mut cx.active_mut().pref;

		if !opt.interactive {
			pref.name = opt.name.unwrap_or_default().into_owned();
			succ!(render!());
		}

		let mut input = InputProxy::show(
			InputCfg::tab_rename().with_value(opt.name.unwrap_or(Cow::Borrowed(&pref.name))),
		);
		tokio::spawn(async move {
			if let Some(Ok(name)) = input.recv().await {
				MgrProxy::tab_rename(tab, name);
			}
		});
		succ!();
	}
}
