use yazi_config::keymap::Exec;

use crate::tab::Tab;

pub struct Opt(Option<bool>);

impl From<Option<bool>> for Opt {
	fn from(b: Option<bool>) -> Self { Self(b) }
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self(match e.named.get("state").map(|s| s.as_bytes()) {
			Some(b"true") => Some(true),
			Some(b"false") => Some(false),
			_ => None,
		})
	}
}

impl Tab {
	pub fn select(&mut self, opt: impl Into<Opt>) -> bool {
		if let Some(u) = self.current.hovered().map(|h| h.url()) {
			return self.current.files.select(&u, opt.into().0);
		}
		false
	}

	pub fn select_all(&mut self, opt: impl Into<Opt>) -> bool {
		self.current.files.select_all(opt.into().0)
	}
}
