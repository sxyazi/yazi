use yazi_macro::render;
use yazi_parser::{mgr::TabCreateOpt, tab::CdSource};
use yazi_proxy::AppProxy;

use crate::{mgr::Tabs, tab::Tab};

const MAX_TABS: usize = 9;

impl Tabs {
	#[yazi_codegen::command]
	pub fn create(&mut self, opt: TabCreateOpt) {
		if self.items.len() >= MAX_TABS {
			AppProxy::notify_warn("Too many tabs", "You can only open up to 9 tabs at the same time.");
			return;
		}

		let mut tab = Tab::default();
		if let Some(wd) = opt.wd {
			tab.cd((wd, CdSource::Tab));
		} else if let Some(h) = self.active().hovered() {
			tab.pref = self.active().pref.clone();
			tab.apply_files_attrs();
			tab.reveal((h.url.to_regular(), CdSource::Tab));
		} else {
			tab.pref = self.active().pref.clone();
			tab.apply_files_attrs();
			tab.cd((self.active().cwd().to_regular(), CdSource::Tab));
		}

		self.items.insert(self.cursor + 1, tab);
		self.set_idx(self.cursor + 1);
		render!();
	}
}
