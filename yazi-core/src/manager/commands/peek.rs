use yazi_shared::{event::Exec, fs::Url, MIME_DIR};

use crate::manager::Manager;

#[derive(Debug, Default)]
pub struct Opt {
	skip:        Option<usize>,
	only_if:     Option<Url>,
	upper_bound: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self {
			skip:        e.args.first().and_then(|s| s.parse().ok()),
			only_if:     e.named.get("only-if").map(Url::from),
			upper_bound: e.named.contains_key("upper-bound"),
		}
	}
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Default::default() }
}

impl Manager {
	pub fn peek(&mut self, opt: impl Into<Opt>) -> bool {
		let Some(hovered) = self.hovered() else {
			return self.active_mut().preview.reset();
		};

		let opt = opt.into() as Opt;
		if matches!(opt.only_if, Some(ref u) if *u != hovered.url) {
			return false;
		}

		let mime = if hovered.is_dir() {
			MIME_DIR.to_owned()
		} else if let Some(s) = self.mimetype.get(&hovered.url) {
			s.to_owned()
		} else {
			return self.active_mut().preview.reset();
		};

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

		self.active_mut().preview.go(hovered, mime);
		false
	}
}
