use tokio::fs;
use yazi_config::keymap::{Exec, KeymapLayer};

use crate::{completion::Completion, emit};

pub struct Opt<'a> {
	word:   &'a str,
	ticket: usize,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self {
		Self {
			word:   e.args.first().map(|w| w.as_str()).unwrap_or_default(),
			ticket: e.named.get("ticket").and_then(|v| v.parse().ok()).unwrap_or(0),
		}
	}
}

impl Completion {
	pub fn trigger<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into();
		if self.ticket >= opt.ticket {
			return false;
		}

		self.close(false);
		self.ticket = opt.ticket;

		let word = opt.word.to_owned();
		let ticket = self.ticket;
		tokio::spawn(async move {
			let (parent, child) = word.rsplit_once('/').unwrap_or((".", word.as_str()));

			let mut dir = fs::read_dir(parent).await?;
			let mut cands = Vec::new();
			while let Ok(Some(f)) = dir.next_entry().await {
				let name = f.file_name().to_string_lossy().into_owned();
				if !name.starts_with(child) {
					continue;
				}

				cands.push(name);
				if cands.len() >= 20 {
					break;
				}
			}

			if !cands.is_empty() {
				emit!(Call(
					Exec::call("show", cands).with("ticket", ticket).vec(),
					KeymapLayer::Completion
				));
			}

			Ok::<(), anyhow::Error>(())
		});
		false
	}
}
