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
	pub fn select_all(&mut self, opt: impl Into<Opt>) {
		let mut b = false;
		match opt.into().state {
			Some(true) => {
				for f in self.current.files.iter() {
					b |= self.selected.add(&f.url);
				}
			}
			Some(false) => {
				for f in self.current.files.iter() {
					b |= self.selected.remove(&f.url);
				}
			}
			None => {
				for f in self.current.files.iter() {
					b |= self.selected.remove(&f.url) || self.selected.add(&f.url);
				}
			}
		}
		render!(b);
	}
}
