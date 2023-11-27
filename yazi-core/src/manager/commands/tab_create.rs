use yazi_config::keymap::Exec;
use yazi_shared::fs::Url;

use crate::{manager::Tabs, tab::Tab};

const MAX_TABS: usize = 9;

pub struct Opt {
	url:     Option<Url>,
	current: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		let mut opt = Self { url: None, current: e.named.contains_key("current") };

		if !opt.current {
			opt.url = Some(e.args.first().map_or_else(|| Url::from("."), Url::from));
		}
		opt
	}
}

impl Tabs {
	pub fn create(&mut self, opt: impl Into<Opt>) -> bool {
		if self.items.len() >= MAX_TABS {
			return false;
		}

		let opt = opt.into() as Opt;
		let url = if opt.current { self.active().current.cwd.to_owned() } else { opt.url.unwrap() };

		let mut tab = Tab::from(url);
		tab.conf = self.active().conf.clone();
		tab.apply_files_attrs(false);

		self.items.insert(self.idx + 1, tab);
		self.set_idx(self.idx + 1);
		true
	}
}
