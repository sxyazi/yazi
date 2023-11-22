use yazi_config::{keymap::{Exec, KeymapLayer}, MANAGER};
use yazi_shared::{Url, MIME_DIR};

use crate::{emit, manager::Manager};

pub struct Opt {
	step:        isize,
	sequent:     Option<Url>,
	upper_bound: Option<usize>,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self {
			step:        e.args.first().and_then(|s| s.parse().ok()).unwrap_or(0),
			sequent:     e.named.get("sequent").map(Url::from),
			upper_bound: e.named.get("upper-bound").and_then(|s| s.parse().ok()),
		}
	}
}
impl From<isize> for Opt {
	fn from(step: isize) -> Self { Self { step, sequent: None, upper_bound: None } }
}

impl Manager {
	#[inline]
	pub fn _peek_upper_bound(bound: usize, sequent: &Url) {
		emit!(Call(
			Exec::call("peek", vec![])
				.with("sequent", sequent.to_string())
				.with("upper-bound", bound.to_string())
				.vec(),
			KeymapLayer::Manager
		));
	}

	pub fn peek(&mut self, opt: impl Into<Opt>) -> bool {
		let Some(hovered) = self.hovered() else {
			return self.active_mut().preview.reset(|_| true);
		};

		let opt = opt.into() as Opt;
		if matches!(opt.sequent, Some(ref u) if *u != hovered.url) {
			return false;
		}

		if hovered.is_dir() {
			return self.peek_folder(opt, hovered.url.clone());
		}

		let Some(mime) = self.mimetype.get(&hovered.url).cloned() else {
			return self.active_mut().preview.reset(|_| true);
		};

		let url = hovered.url.clone();
		self.active_mut().preview.arrow(opt.step, &mime);
		if let Some(bound) = opt.upper_bound {
			self.active_mut().preview.apply_bound(bound);
		}

		self.active_mut().preview.go(&url, &mime);
		false
	}

	fn peek_folder(&mut self, opt: Opt, url: Url) -> bool {
		let (skip, bound) = self
			.active()
			.history
			.get(&url)
			.map(|f| (f.offset, f.files.len().saturating_sub(MANAGER.layout.folder_height())))
			.unwrap_or_default();

		if opt.sequent.is_some() {
			self.active_mut().preview.arrow(opt.step, MIME_DIR);
		} else {
			self.active_mut().preview.set_skip(skip);
		}

		self.active_mut().preview.apply_bound(bound);
		self.active_mut().preview.go_folder(url, opt.sequent.is_none());
		false
	}
}
