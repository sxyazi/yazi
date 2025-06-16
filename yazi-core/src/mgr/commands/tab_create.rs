use yazi_boot::BOOT;
use yazi_fs::expand_path;
use yazi_macro::render;
use yazi_proxy::AppProxy;
use yazi_shared::{event::CmdCow, url::Url};

use crate::{mgr::Tabs, tab::{Tab, commands::CdSource}};

const MAX_TABS: usize = 9;

struct Opt {
	wd: Option<Url>,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self {
		if c.bool("current") {
			return Self { wd: None };
		}
		let Some(mut wd) = c.take_first_url() else {
			return Self { wd: Some(Url::from(&BOOT.cwds[0])) };
		};
		if wd.is_regular() && !c.bool("raw") {
			wd = Url::from(expand_path(wd));
		}
		Self { wd: Some(wd) }
	}
}

impl Tabs {
	#[yazi_codegen::command]
	pub fn create(&mut self, opt: Opt) {
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
