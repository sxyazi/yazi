use yazi_config::keymap::Exec;

use crate::tab::Tab;

pub struct Opt {
	state: Option<bool>,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self {
			state: match e.named.get("state").map(|s| s.as_bytes()) {
				Some(b"true") => Some(true),
				Some(b"false") => Some(false),
				_ => None,
			},
		}
	}
}
impl From<Option<bool>> for Opt {
	fn from(state: Option<bool>) -> Self { Self { state } }
}

impl Tab {
	pub fn select(&mut self, opt: impl Into<Opt>) -> bool {
		if let Some(u) = self.current.hovered().map(|h| h.url()) {
			return self.current.files.select(&u, opt.into().state);
		}
		false
	}

	pub fn select_all(&mut self, opt: impl Into<Opt>) -> bool {
		self.current.files.select_all(opt.into().state)
	}
}
