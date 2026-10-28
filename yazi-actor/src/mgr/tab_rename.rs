use std::borrow::Cow;

use anyhow::Result;
use yazi_config::YAZI;
use yazi_core::Ctx;
use yazi_macro::{render, succ};
use yazi_parser::mgr::TabRenameForm;
use yazi_proxy::MgrProxy;
use yazi_shared::{data::Data, scope::ScopeLike};
use yazi_widgets::input::InputEvent;

use crate::{Actor, act, input};

pub struct TabRename;

impl Actor for TabRename {
	type Form = TabRenameForm;

	const NAME: &str = "tab_rename";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		if !form.interactive {
			cx.tab_mut().pref.name = form.name.unwrap_or_default().into_owned();
			act!(app:title, cx).ok();
			succ!(render!());
		}

		let scope = cx.tab().to_scope();
		let mut input = input!(
			cx,
			YAZI.input.tab_rename().with_value(form.name.unwrap_or(Cow::Borrowed(&cx.tab().pref.name)))
		)?;

		tokio::spawn(async move {
			if let Some(InputEvent::Submit(name)) = input.recv().await {
				MgrProxy::tab_rename(scope, name);
			}
		});
		succ!();
	}
}
