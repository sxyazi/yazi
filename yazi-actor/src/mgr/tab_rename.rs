use std::borrow::Cow;

use anyhow::Result;
use yazi_config::popup::InputCfg;
use yazi_macro::{act, input, render, succ};
use yazi_parser::mgr::TabRenameForm;
use yazi_proxy::MgrProxy;
use yazi_shared::data::Data;
use yazi_widgets::input::InputEvent;

use crate::{Actor, Ctx};

pub struct TabRename;

impl Actor for TabRename {
	type Form = TabRenameForm;

	const NAME: &str = "tab_rename";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let tab = cx.tab().id;
		let pref = &mut cx.tab_mut().pref;

		if !form.interactive {
			pref.name = form.name.unwrap_or_default().into_owned();
			act!(app:title, cx).ok();
			succ!(render!());
		}

		let mut input = input!(
			cx,
			InputCfg::tab_rename().with_value(form.name.unwrap_or(Cow::Borrowed(&pref.name)))
		)?;

		tokio::spawn(async move {
			if let Some(InputEvent::Submit(name)) = input.recv().await {
				MgrProxy::tab_rename(tab, name);
			}
		});
		succ!();
	}
}
