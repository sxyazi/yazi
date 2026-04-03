use std::borrow::Cow;

use anyhow::Result;
use yazi_config::popup::InputCfg;
use yazi_macro::{act, succ};
use yazi_parser::mgr::ShellForm;
use yazi_proxy::{InputProxy, TasksProxy};
use yazi_scheduler::process::ProcessOpt;
use yazi_shared::data::Data;
use yazi_widgets::input::InputEvent;

use crate::{Actor, Ctx};

pub struct Shell;

impl Actor for Shell {
	type Form = ShellForm;

	const NAME: &str = "shell";

	fn act(cx: &mut Ctx, mut form: Self::Form) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		let cwd = form.cwd.take().unwrap_or(cx.cwd().into()).into_owned();
		let selected: Vec<_> = cx.tab().hovered_and_selected().cloned().map(Into::into).collect();

		let input = form.interactive.then(|| {
			InputProxy::show(InputCfg::shell(form.block).with_value(&*form.run).with_cursor(form.cursor))
		});

		tokio::spawn(async move {
			if let Some(mut rx) = input {
				match rx.recv().await {
					Some(InputEvent::Submit(e)) => form.run = Cow::Owned(e),
					_ => return,
				}
			}
			if form.run.is_empty() {
				return;
			}

			TasksProxy::open_shell_compat(ProcessOpt {
				cwd:    cwd.into(),
				cmd:    form.run.to_string().into(),
				args:   selected,
				block:  form.block,
				orphan: form.orphan,
				spread: true,
			});
		});

		succ!();
	}
}
