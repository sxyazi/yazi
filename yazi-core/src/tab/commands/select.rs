use yazi_shared::{event::Cmd, render};

use crate::tab::Tab;

pub struct Opt {
	state: Option<bool>,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self {
		Self {
			state: match c.named.get("state").map(|s| s.as_str()) {
				Some("true") => Some(true),
				Some("false") => Some(false),
				_ => None,
			},
		}
	}
}
impl From<Option<bool>> for Opt {
	fn from(state: Option<bool>) -> Self { Self { state } }
}

impl Tab {
	pub fn select(&mut self, opt: impl Into<Opt>) {
		if let Some(u) = self.current.hovered().map(|h| h.url()) {
			render!(self.current.files.select(&u, opt.into().state));
		}
	}

	pub fn select_all(&mut self, opt: impl Into<Opt>) {
		render!(self.current.files.select_all(opt.into().state));
	}
}
