use std::iter;

use yazi_parser::mgr::WatchOpt;

use crate::mgr::Mgr;

impl Mgr {
	#[yazi_codegen::command]
	pub fn watch(&mut self, _: Opt) {
		let it = iter::once(self.tabs.active().cwd())
			.chain(self.tabs.parent().map(|p| &p.url))
			.chain(self.tabs.hovered().filter(|h| h.is_dir()).map(|h| &h.url));

		self.watcher.watch(it);
	}
}
