use yazi_shared::{event::Exec, fs::Url, render};

use crate::{manager::Tabs, tab::Tab};

const MAX_TABS: usize = 9;

pub struct Opt {
	url:     Option<Url>,
	current: bool,
}

impl From<Exec> for Opt {
	fn from(mut e: Exec) -> Self {
		let mut opt = Self { url: None, current: e.named.contains_key("current") };

		if !opt.current {
			opt.url = Some(e.take_first().map_or_else(|| Url::from("."), Url::from));
		}
		opt
	}
}

impl Tabs {
	pub fn create(&mut self, opt: impl Into<Opt>) {
		if self.items.len() >= MAX_TABS {
			return;
		}

		let opt = opt.into() as Opt;
		let url = if opt.current { self.active().current.cwd.to_owned() } else { opt.url.unwrap() };

		let mut tab = Tab::from(url);
		tab.conf = self.active().conf.clone();
		tab.apply_files_attrs();

		self.items.insert(self.idx + 1, tab);
		self.set_idx(self.idx + 1);
		render!();
	}
}
