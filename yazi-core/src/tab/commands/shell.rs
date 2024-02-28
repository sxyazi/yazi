use yazi_config::{open::Opener, popup::InputCfg};
use yazi_shared::event::Cmd;

use crate::{input::Input, tab::Tab, tasks::Tasks};

pub struct Opt {
	exec:    String,
	block:   bool,
	confirm: bool,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			exec:    c.take_first().unwrap_or_default(),
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
			if !opt.confirm || opt.exec.is_empty() {
				let mut result = Input::_show(InputCfg::shell(opt.block).with_value(opt.exec));
				match result.recv().await {
					Some(Ok(e)) => opt.exec = e,
					_ => return,
				}
			}

			Tasks::_open_with(selected, Opener {
				exec:   opt.exec,
				block:  opt.block,
				orphan: false,
				desc:   Default::default(),
				for_:   None,
				spread: true,
			});
		});
	}
}
