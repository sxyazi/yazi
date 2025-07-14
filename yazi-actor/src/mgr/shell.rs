use std::borrow::Cow;

use anyhow::Result;
use yazi_config::{opener::OpenerRule, popup::InputCfg};
use yazi_macro::{act, succ};
use yazi_parser::tab::ShellOpt;
use yazi_proxy::{InputProxy, TasksProxy};
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Shell;

impl Actor for Shell {
	type Options = ShellOpt;

	const NAME: &'static str = "shell";

	fn act(cx: &mut Ctx, mut opt: Self::Options) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		let cwd = opt.cwd.take().unwrap_or_else(|| cx.cwd().clone());
		let selected = cx.tab().hovered_and_selected().cloned().collect();

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

			TasksProxy::open_with(
				Cow::Owned(OpenerRule {
					run:    opt.run.into_owned(),
					block:  opt.block,
					orphan: opt.orphan,
					desc:   Default::default(),
					r#for:  None,
					spread: true,
				}),
				cwd,
				selected,
			);
		});

		succ!();
	}
}
