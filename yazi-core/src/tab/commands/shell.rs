use std::{borrow::Cow, fmt::Display};

use yazi_config::{opener::OpenerRule, popup::InputCfg};
use yazi_parser::tab::ShellOpt;
use yazi_proxy::{AppProxy, InputProxy, TasksProxy};

use crate::tab::Tab;

impl Tab {
	pub fn shell(&mut self, opt: impl TryInto<ShellOpt, Error = impl Display>) {
		if !self.try_escape_visual() {
			return;
		}

		let mut opt = match opt.try_into() {
			Ok(o) => o as ShellOpt,
			Err(e) => return AppProxy::notify_warn("`shell` command", e),
		};

		let cwd = opt.cwd.take().unwrap_or_else(|| self.cwd().clone());
		let selected = self.hovered_and_selected().cloned().collect();

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
	}
}
