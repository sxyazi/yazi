use yazi_config::open::Opener;

use crate::{emit, input::InputOpt, tab::Tab};

impl Tab {
	pub fn shell(&self, exec: &str, block: bool, confirm: bool) -> bool {
		let selected: Vec<_> = self
			.selected()
			.into_iter()
			.map(|f| (f.url.as_os_str().to_owned(), Default::default()))
			.collect();

		let mut exec = exec.to_owned();
		tokio::spawn(async move {
			if !confirm || exec.is_empty() {
				let mut result = emit!(Input(
					InputOpt::top(if block { "Shell (block):" } else { "Shell:" })
						.with_value(&exec)
						.with_highlight()
				));
				match result.recv().await {
					Some(Ok(e)) => exec = e,
					_ => return,
				}
			}

			emit!(Open(
				selected,
				Some(Opener {
					exec,
					block,
					orphan: false,
					desc: Default::default(),
					for_: None,
					spread: true
				})
			));
		});

		false
	}
}
