use yazi_boot::BOOT;
use yazi_proxy::AppProxy;
use yazi_shared::{event::{Cmd, Data}, fs::Url, render};

use crate::{manager::Tabs, tab::Tab};

const MAX_TABS: usize = 9;

pub struct Opt {
	url:     Url,
	current: bool,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		if c.bool("current") {
			Self { url: Default::default(), current: true }
		} else {
			Self {
				url:     c
					.take_first()
					.and_then(Data::into_url)
					.unwrap_or_else(|| Url::from(&BOOT.cwds[0])),
				current: false,
			}
		}
	}
}

impl Tabs {
	pub fn create(&mut self, opt: impl Into<Opt>) {
		if self.items.len() >= MAX_TABS {
			AppProxy::notify_warn("Too many tabs", "You can only open up to 9 tabs at the same time.");
			return;
		}

		let opt = opt.into() as Opt;
		let mut tab = Tab { idx: self.cursor + 1, ..Default::default() };

		if !opt.current {
			tab.cd(opt.url);
		} else if let Some(h) = self.active().current.hovered() {
			tab.conf = self.active().conf.clone();
			tab.apply_files_attrs();
			tab.reveal(h.url_owned());
		} else {
			tab.conf = self.active().conf.clone();
			tab.apply_files_attrs();
			tab.cd(self.active().cwd().clone());
		}

		self.items.insert(self.cursor + 1, tab);
		self.set_idx(self.cursor + 1);
		self.reorder();
		render!();
	}
}
