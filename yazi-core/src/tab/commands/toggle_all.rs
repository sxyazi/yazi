use yazi_macro::render;
use yazi_parser::tab::ToggleAllOpt;
use yazi_proxy::AppProxy;

use crate::tab::Tab;

impl Tab {
	#[yazi_codegen::command]
	pub fn toggle_all(&mut self, opt: ToggleAllOpt) {
		use yazi_shared::Either::*;

		let it = self.current.files.iter().map(|f| &f.url);
		let either = match opt.state {
			Some(true) if opt.urls.is_empty() => Left((vec![], it.collect())),
			Some(true) => Right((vec![], opt.urls)),
			Some(false) if opt.urls.is_empty() => Left((it.collect(), vec![])),
			Some(false) => Right((opt.urls, vec![])),
			None if opt.urls.is_empty() => Left(it.partition(|&u| self.selected.contains_key(u))),
			None => Right(opt.urls.into_iter().partition(|u| self.selected.contains_key(u))),
		};

		let warn = match either {
			Left((removal, addition)) => {
				render!(self.selected.remove_many(&removal) > 0);
				render!(self.selected.add_many(&addition), > 0) != addition.len()
			}
			Right((removal, addition)) => {
				render!(self.selected.remove_many(&removal) > 0);
				render!(self.selected.add_many(&addition), > 0) != addition.len()
			}
		};

		if warn {
			AppProxy::notify_warn(
				"Toggle all",
				"Some files cannot be selected, due to path nesting conflict.",
			);
		}
	}
}
