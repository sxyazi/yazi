use std::borrow::Cow;

use yazi_config::{open::Opener, popup::InputCfg};
use yazi_proxy::{AppProxy, InputProxy, TasksProxy};
use yazi_shared::event::Cmd;

use crate::tab::Tab;

pub struct Opt {
	run:         String,
	block:       bool,
	orphan:      bool,
	confirm:     bool,
	interactive: bool,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			run:         c.take_first_str().unwrap_or_default(),
			block:       c.bool("block"),
			orphan:      c.bool("orphan"),
			confirm:     c.bool("confirm"),
			interactive: c.bool("interactive"),
		}
	}
}

impl Tab {
	pub fn shell(&mut self, opt: impl Into<Opt>) {
		if !self.try_escape_visual() {
			return;
		}

		let mut opt = opt.into() as Opt;

		// TODO: Remove in v0.3.2
		if !opt.interactive && !opt.confirm {
			AppProxy::notify_error(
				"`shell` command",
				r#"WARNING: In Yazi v0.3, the behavior of the interactive `shell` (i.e., shell templates) must be explicitly specified with either `--interactive` or `--confirm`.

Please replace e.g. `shell` with `shell --interactive`, `shell "my-template"` with `shell "my-template" --interactive`, in your keymap.toml"#,
			);
			return;
		} else if opt.interactive && opt.confirm {
			AppProxy::notify_error(
				"`shell` command",
				"The `shell` command cannot specify both `--confirm` and `--interactive` at the same time.",
			);
			return;
		}

		let selected = self.hovered_and_selected(true).cloned().collect();

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
					orphan: opt.orphan,
					desc:   Default::default(),
					for_:   None,
					spread: true,
				}),
			);
		});
	}
}
