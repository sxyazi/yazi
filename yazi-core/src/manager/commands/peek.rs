use yazi_config::MANAGER;
use yazi_shared::{fs::Url, Exec, Layer, MIME_DIR};

use crate::{emit, manager::Manager};

#[derive(Debug)]
pub struct Opt {
	step:        isize,
	only_if:     Option<Url>,
	upper_bound: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self {
			step:        e.args.first().and_then(|s| s.parse().ok()).unwrap_or(0),
			only_if:     e.named.get("only-if").map(Url::from),
			upper_bound: e.named.contains_key("upper-bound"),
		}
	}
}
impl From<isize> for Opt {
	fn from(step: isize) -> Self { Self { step, only_if: None, upper_bound: false } }
}

impl Manager {
	#[inline]
	pub fn _peek_upper_bound(bound: usize, only_if: &Url) {
		emit!(Call(
			Exec::call("peek", vec![bound.to_string()])
				.with("only-if", only_if.to_string())
				.with_bool("upper-bound", true)
				.vec(),
			Layer::Manager
		));
	}

	pub fn peek(&mut self, opt: impl Into<Opt>) -> bool {
		let Some(hovered) = self.hovered() else {
			return self.active_mut().preview.reset();
		};

		let opt = opt.into() as Opt;
		if matches!(opt.only_if, Some(ref u) if *u != hovered.url) {
			return false;
		}

		if hovered.is_dir() {
			return self.peek_folder(opt, hovered.url.clone());
		}

		let Some(mime) = self.mimetype.get(&hovered.url).cloned() else {
			return self.active_mut().preview.reset();
		};

		let (url, cha) = (hovered.url.clone(), hovered.cha);
		if opt.upper_bound {
			self.active_mut().preview.arrow(0, &mime, Some(opt.step as usize));
		} else if self.active().preview.same_url(&url) {
			self.active_mut().preview.arrow(opt.step, &mime, None);
		} else {
			self.active_mut().preview.arrow(0, &mime, Some(0));
			self.active_mut().preview.reset();
		}

		self.active_mut().preview.go(&url, cha, &mime);
		false
	}

	fn peek_folder(&mut self, opt: Opt, url: Url) -> bool {
		let folder = self.active().history.get(&url);
		let (skip, bound) = folder
			.map(|f| (f.offset, f.files.len().saturating_sub(MANAGER.layout.folder_height())))
			.unwrap_or_default();

		let in_chunks = folder.is_none();
		if self.active().preview.same_url(&url) {
			self.active_mut().preview.arrow(opt.step, MIME_DIR, Some(bound));
			self.active_mut().preview.sync_skip()
		} else {
			self.active_mut().preview.arrow(skip as isize, MIME_DIR, Some(skip));
			self.active_mut().preview.go_folder(url, in_chunks);
			false
		}
	}
}
