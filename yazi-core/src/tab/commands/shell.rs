use std::{borrow::Cow, fmt::Display};

use anyhow::bail;
use yazi_config::{open::Opener, popup::InputCfg};
use yazi_proxy::{AppProxy, InputProxy, TasksProxy};
use yazi_shared::event::{CmdCow, Data};

use crate::tab::Tab;

pub struct Opt {
	run:         Cow<'static, str>,
	block:       bool,
	orphan:      bool,
	interactive: bool,
	cursor:      Option<usize>,
}

impl TryFrom<CmdCow> for Opt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let me = Self {
			run:         c.take_first_str().unwrap_or_default(),
			block:       c.bool("block"),
			orphan:      c.bool("orphan"),
			interactive: c.bool("interactive"),
			cursor:      c.get("cursor").and_then(Data::as_usize),
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

		let selected = self.hovered_and_selected(true).cloned().collect();
		tokio::spawn(async move {
			if opt.interactive {
				let mut result =
					InputProxy::show(InputCfg::shell(opt.block).with_value(opt.run).with_cursor(opt.cursor));
				match result.recv().await {
					Some(Ok(e)) => opt.run = Cow::Owned(e),
					_ => return,
				}
			}
			if opt.run.is_empty() {
				return;
			}

			TasksProxy::open_with(
				selected,
				Cow::Owned(Opener {
					run:    opt.run.into_owned(),
					block:  opt.block,
					orphan: opt.orphan,
					desc:   Default::default(),
					for_:   None,
					spread: true,
				}),
			);
		});
	}
}
