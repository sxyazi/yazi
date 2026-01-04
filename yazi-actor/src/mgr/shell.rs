use std::borrow::Cow;

use anyhow::Result;
use yazi_config::popup::InputCfg;
use yazi_fs::Splatter;
use yazi_macro::{act, succ};
use yazi_parser::{mgr::ShellOpt, tasks::ProcessOpenOpt};
use yazi_proxy::{InputProxy, TasksProxy};
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Shell;

impl Actor for Shell {
	type Options = ShellOpt;

	const NAME: &str = "shell";

	fn act(cx: &mut Ctx, mut opt: Self::Options) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		let cwd = opt.cwd.take().unwrap_or(cx.cwd().into()).into_owned();
		let selected: Vec<_> = cx.tab().hovered_and_selected().cloned().map(Into::into).collect();

		let input = opt.interactive.then(|| {
			InputProxy::show(InputCfg::shell(opt.block).with_value(&*opt.run).with_cursor(opt.cursor))
		});

		tokio::spawn(async move {
			if let Some(mut rx) = input {
				match rx.recv().await {
					Some(Ok(e)) => opt.run = Cow::Owned(e),
					_ => return,
				}
			}
			if opt.run.is_empty() {
				return;
			}

			TasksProxy::open_shell_compat(ProcessOpenOpt {
				cwd:    cwd.into(),
				cmd:    Splatter::new(&selected).splat(&*opt.run),
				args:   selected,
				block:  opt.block,
				orphan: opt.orphan,
				done:   None,
				spread: true,
			});
		});

		succ!();
	}
}
