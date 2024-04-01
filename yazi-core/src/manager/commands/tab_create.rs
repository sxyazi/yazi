use yazi_shared::{event::Cmd, fs::Url, render};

use crate::{manager::Tabs, tab::Tab};

const MAX_TABS: usize = 9;

pub struct Opt {
	url:     Option<Url>,
	current: bool,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		let mut opt = Self { url: None, current: c.named.contains_key("current") };

		if !opt.current {
			opt.url = Some(c.take_first().map_or_else(|| Url::from("."), Url::from));
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

		let mut tab = Tab::default();
		tab.conf = self.active().conf.clone();
		tab.apply_files_attrs();
		tab.cd(url);

		self.items.insert(self.cursor + 1, tab);
		self.set_idx(self.cursor + 1);
		self.reorder();
		render!();
	}
}
