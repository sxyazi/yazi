use std::borrow::Cow;

use yazi_config::{open::Opener, popup::InputCfg};
use yazi_proxy::{InputProxy, TasksProxy};
use yazi_shared::event::Cmd;

use crate::tab::Tab;

pub struct Opt {
	run:     String,
	block:   bool,
	confirm: bool,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			run:     c.take_first().unwrap_or_default(),
			block:   c.named.contains_key("block"),
			confirm: c.named.contains_key("confirm"),
		}
	}
}

impl Tab {
	pub fn shell(&mut self, opt: impl Into<Opt>) {
		if !self.try_escape_visual() {
			return;
		}

		let mut opt = opt.into() as Opt;
		let selected = self.hovered_and_selected().into_iter().cloned().collect();

		tokio::spawn(async move {
			if !opt.confirm || opt.run.is_empty() {
				let mut result = InputProxy::show(InputCfg::shell(opt.block).with_value(opt.run));
				match result.recv().await {
					Some(Ok(e)) => opt.run = e,
					_ => return,
				}
			}

			TasksProxy::open_with(
				selected,
				Cow::Owned(Opener {
					run:    opt.run,
					block:  opt.block,
					orphan: false,
					desc:   Default::default(),
					for_:   None,
					spread: true,
				}),
			);
		});
	}
}
