use yazi_shared::{emit, event::Exec, fs::Url, render, Layer};

use crate::manager::Manager;

#[derive(Debug, Default)]
pub struct Opt {
	skip:        Option<usize>,
	force:       bool,
	only_if:     Option<Url>,
	upper_bound: bool,
}

impl From<Exec> for Opt {
	fn from(mut e: Exec) -> Self {
		Self {
			skip:        e.take_first().and_then(|s| s.parse().ok()),
			force:       e.named.contains_key("force"),
			only_if:     e.take_name("only-if").map(Url::from),
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
		emit!(Call(Exec::call("peek", vec![]).with_bool("force", force), Layer::Manager));
	}

	pub fn peek(&mut self, opt: impl Into<Opt>) {
		let Some(hovered) = self.hovered() else {
			return render!(self.active_mut().preview.reset());
		};

		let hovered = hovered.clone();
		if !self.active().preview.same_url(&hovered.url) {
			self.active_mut().preview.skip = 0;
			render!(self.active_mut().preview.reset());
		}

		let opt = opt.into() as Opt;
		if matches!(opt.only_if, Some(ref u) if *u != hovered.url) {
			return;
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
			let mtime = self.active().history.get(&hovered.url).and_then(|f| f.mtime);
			self.active_mut().preview.go_folder(hovered, mtime, opt.force);
			return;
		}

		if let Some(m) = self.mimetype.get(&hovered.url).cloned() {
			self.active_mut().preview.go(hovered, &m, opt.force);
		} else {
			render!(self.active_mut().preview.reset());
		}
	}
}
