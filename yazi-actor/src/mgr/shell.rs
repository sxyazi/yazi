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

	fn act(cx: &mut Ctx, mut opt: Self::Form) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		let cwd = opt.cwd.take().unwrap_or(cx.cwd().into()).into_owned();
		let selected: Vec<_> = cx.tab().hovered_and_selected().cloned().map(Into::into).collect();

		let input = opt.interactive.then(|| {
			InputProxy::show(InputCfg::shell(opt.block).with_value(&*opt.run).with_cursor(opt.cursor))
		});

		tokio::spawn(async move {
			if let Some(mut rx) = input {
				match rx.recv().await {
					Some(InputEvent::Submit(e)) => opt.run = Cow::Owned(e),
					_ => return,
				}
			}
			if opt.run.is_empty() {
				return;
			}

			TasksProxy::open_shell_compat(ProcessOpt {
				cwd:    cwd.into(),
				cmd:    opt.run.to_string().into(),
				args:   selected,
				block:  opt.block,
				orphan: opt.orphan,
				spread: true,
			});
		});

		succ!();
	}
}
