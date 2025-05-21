use std::{borrow::Cow, fmt::Display};

use anyhow::bail;
use yazi_config::{opener::OpenerRule, popup::InputCfg};
use yazi_proxy::{AppProxy, InputProxy, TasksProxy};
use yazi_shared::{event::{CmdCow, Data}, url::Url};

use crate::tab::Tab;

pub struct Opt {
	run: Cow<'static, str>,
	cwd: Option<Url>,

	block:       bool,
	orphan:      bool,
	interactive: bool,

	cursor: Option<usize>,
}

impl TryFrom<CmdCow> for Opt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let me = Self {
			run: c.take_first_str().unwrap_or_default(),
			cwd: c.take_url("cwd"),

			block:       c.bool("block"),
			orphan:      c.bool("orphan"),
			interactive: c.bool("interactive"),

			cursor: c.get("cursor").and_then(Data::as_usize),
		};

		if me.cursor.is_some_and(|c| c > me.run.chars().count()) {
			bail!("The cursor position is out of bounds.");
		}

		Ok(me)
	}
}

impl Tab {
	pub fn shell(&mut self, opt: impl TryInto<Opt, Error = impl Display>) {
		if !self.try_escape_visual() {
			return;
		}

		let mut opt = match opt.try_into() {
			Ok(o) => o as Opt,
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
