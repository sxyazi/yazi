use yazi_shared::{event::Cmd, render};

use crate::{notify::Notify, tab::Tab};

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
		let iter = self.current.files.iter().map(|f| &f.url);
		let (removal, addition): (Vec<_>, Vec<_>) = match opt.into().state {
			Some(true) => (vec![], iter.collect()),
			Some(false) => (iter.collect(), vec![]),
			None => iter.partition(|&u| self.selected.contains(u)),
		};

		render!(self.selected.remove_many(&removal) > 0);
		let added = self.selected.add_many(&addition);

		render!(added > 0);
		if added != addition.len() {
			Notify::_push_warn(
				"Select all",
				"Some files cannot be selected, due to path nesting conflict.",
			);
		}
	}
}
