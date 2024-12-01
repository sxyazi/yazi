use yazi_macro::render;
use yazi_proxy::AppProxy;
use yazi_shared::event::CmdCow;

use crate::tab::Tab;

struct Opt {
	state: Option<bool>,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self {
			state: match c.str("state") {
				Some("on") => Some(true),
				Some("off") => Some(false),
				_ => None,
			},
		}
	}
}
impl From<Option<bool>> for Opt {
	fn from(state: Option<bool>) -> Self { Self { state } }
}

impl Tab {
	#[yazi_codegen::command]
	pub fn toggle_all(&mut self, opt: Opt) {
		let iter = self.current.files.iter().map(|f| &f.url);
		let (removal, addition): (Vec<_>, Vec<_>) = match opt.state {
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
				"Toggle all",
				"Some files cannot be selected, due to path nesting conflict.",
			);
		}
	}
}
