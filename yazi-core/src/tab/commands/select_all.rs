use yazi_proxy::AppProxy;
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
		let state = opt.into().state;
		if state == Some(false) {
			return render!(self.selected.clear());
		}

		let iter = self.current.files.iter().map(|f| &f.url);
		let (removal, addition): (Vec<_>, Vec<_>) = if state == Some(true) {
			(vec![], iter.collect())
		} else {
			iter.partition(|&u| self.selected.contains(u))
		};

		let same = !self.current.cwd.is_search();
		render!(self.selected.remove_many(&removal, same) > 0);
		let added = self.selected.add_many(&addition, same);

		render!(added > 0);
		if added != addition.len() {
			AppProxy::warn("Select all", "Some files cannot be selected, due to path nesting conflict.");
		}
	}
}
