use yazi_shared::{emit, event::Exec, fs::Url, Layer, MIME_DIR};

use crate::manager::Manager;

#[derive(Debug, Default)]
pub struct Opt {
	skip:        Option<usize>,
	force:       bool,
	only_if:     Option<Url>,
	upper_bound: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self {
			skip:        e.args.first().and_then(|s| s.parse().ok()),
			force:       e.named.contains_key("force"),
			only_if:     e.named.get("only-if").map(Url::from),
			upper_bound: e.named.contains_key("upper-bound"),
		}
	}
}
impl From<bool> for Opt {
	fn from(force: bool) -> Self { Self { force, ..Default::default() } }
}

impl Manager {
	#[inline]
	pub fn _peek(force: bool) {
		emit!(Call(Exec::call("peek", vec![]).with_bool("force", force).vec(), Layer::Manager));
	}

	pub fn peek(&mut self, opt: impl Into<Opt>) -> bool {
		let Some(hovered) = self.hovered() else {
			return self.active_mut().preview.reset();
		};

		let opt = opt.into() as Opt;
		if matches!(opt.only_if, Some(ref u) if *u != hovered.url) {
			return false;
		}

		let hovered = hovered.clone();
		if !self.active().preview.same_url(&hovered.url) {
			self.active_mut().preview.skip = 0;
			self.active_mut().preview.reset();
		}

		if let Some(skip) = opt.skip {
			let preview = &mut self.active_mut().preview;
			if opt.upper_bound {
				preview.skip = preview.skip.min(skip);
			} else {
				preview.skip = skip;
			}
		}

		if hovered.is_dir() {
			if self.active().history.contains_key(&hovered.url) {
				self.active_mut().preview.go(hovered, MIME_DIR, opt.force);
			} else {
				self.active_mut().preview.go_folder(hovered, opt.force);
			}
			return false;
		}

		if let Some(s) = self.mimetype.get(&hovered.url).cloned() {
			self.active_mut().preview.go(hovered, &s, opt.force);
		} else {
			return self.active_mut().preview.reset();
		}
		false
	}
}
