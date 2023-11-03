use tokio::fs;
use yazi_config::keymap::{Exec, KeymapLayer};

use crate::{completion::Completion, emit};

pub struct Opt<'a> {
	before: &'a str,
	ticket: usize,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self {
		Self {
			before: e.named.get("before").map(|s| s.as_str()).unwrap_or_default(),
			ticket: e.named.get("ticket").and_then(|s| s.parse().ok()).unwrap_or(0),
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

		let (parent, child) = opt.before.rsplit_once('/').unwrap_or((".", opt.before));
		if self.caches.contains_key(parent) {
			return self.show(
				&Exec::call("show", vec![])
					.with("cache-name", parent)
					.with("word", child)
					.with("ticket", opt.ticket),
			);
		}

		let ticket = self.ticket;
		let (parent, child) = (parent.to_owned(), child.to_owned());
		tokio::spawn(async move {
			let mut dir = fs::read_dir(&parent).await?;
			let mut cache = Vec::new();
			while let Ok(Some(f)) = dir.next_entry().await {
				let Ok(meta) = f.metadata().await else {
					continue;
				};

				let sep = if !meta.is_dir() {
					""
				} else if cfg!(windows) {
					"\\"
				} else {
					"/"
				};
				cache.push(format!("{}{sep}", f.file_name().to_string_lossy()));
			}

			if !cache.is_empty() {
				emit!(Call(
					Exec::call("show", cache)
						.with("cache-name", parent)
						.with("word", child)
						.with("ticket", ticket)
						.vec(),
					KeymapLayer::Completion
				));
			}

			Ok::<(), anyhow::Error>(())
		});
		false
	}
}
