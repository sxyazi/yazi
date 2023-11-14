use yazi_config::{keymap::Exec, open::Opener, INPUT};

use crate::{emit, input::InputOpt, tab::Tab};

pub struct Opt {
	cmd:     String,
	block:   bool,
	confirm: bool,
}

impl<'a> From<&'a Exec> for Opt {
	fn from(e: &'a Exec) -> Self {
		Self {
			cmd:     e.args.first().map(|e| e.to_owned()).unwrap_or_default(),
			block:   e.named.contains_key("block"),
			confirm: e.named.contains_key("confirm"),
		}
	}
}

impl Tab {
	pub fn shell(&self, opt: impl Into<Opt>) -> bool {
		let selected: Vec<_> = self
			.selected()
			.into_iter()
			.map(|f| (f.url.as_os_str().to_owned(), Default::default()))
			.collect();

		let mut opt = opt.into() as Opt;
		tokio::spawn(async move {
			if !opt.confirm || opt.cmd.is_empty() {
				let title = if opt.block { "Shell (block):" } else { "Shell:" };
				let mut result = emit!(Input(
					InputOpt::from_cfg(title, &INPUT.shell_position, &INPUT.shell_offset)
						.with_value(opt.cmd)
						.with_highlight()
				));
				match result.recv().await {
					Some(Ok(e)) => opt.cmd = e,
					_ => return,
				}
			}

			emit!(Open(
				selected,
				Some(Opener {
					exec:   opt.cmd,
					block:  opt.block,
					orphan: false,
					desc:   Default::default(),
					for_:   None,
					spread: true,
				})
			));
		});

		false
	}
}
