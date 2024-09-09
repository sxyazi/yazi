use yazi_proxy::AppProxy;
use yazi_shared::{event::Cmd, render};

use crate::tab::Tab;

pub struct Opt {
	state: Option<bool>,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			state: match c.take_str("state").as_deref() {
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
		let iter = self.current.files.iter().map(|f| f.url());
		let (removal, addition): (Vec<_>, Vec<_>) = match opt.into().state {
			Some(true) => (vec![], iter.collect()),
			Some(false) => (iter.collect(), vec![]),
			None => iter.partition(|&u| self.selected.contains_key(u)),
		};

		let same = !self.cwd().is_search();
		render!(self.selected.remove_many(&removal, same) > 0);

		let added = self.selected.add_many(&addition, same);
		render!(added > 0);

		if added != addition.len() {
			AppProxy::notify_warn(
				"Select all",
				"Some files cannot be selected, due to path nesting conflict.",
			);
		}
	}
}
