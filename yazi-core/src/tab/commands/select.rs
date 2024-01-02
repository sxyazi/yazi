use yazi_shared::{event::Exec, render};

use crate::tab::Tab;

pub struct Opt {
	state: Option<bool>,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self {
			state: match e.named.get("state").map(|s| s.as_str()) {
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
