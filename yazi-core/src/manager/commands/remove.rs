use yazi_config::popup::ConfirmCfg;
use yazi_proxy::{ConfirmProxy, ManagerProxy};
use yazi_shared::{event::Cmd, fs::Url};

use crate::{manager::Manager, tasks::Tasks};

pub struct Opt {
	force:       bool,
	permanently: bool,
	hovered:     bool,
	targets:     Vec<Url>,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			force:       c.bool("force"),
			permanently: c.bool("permanently"),
			hovered:     c.bool("hovered"),
			targets:     c.take_any("targets").unwrap_or_default(),
		}
	}
}

impl Manager {
	pub fn remove(&mut self, opt: impl Into<Opt>, tasks: &Tasks) {
		if !self.active_mut().try_escape_visual() {
			return;
		}
		let Some(hovered) = self.hovered().map(|h| &h.url) else {
			return;
		};

		let mut opt = opt.into() as Opt;
		opt.targets = if opt.hovered {
			vec![hovered.clone()]
		} else {
			self.selected_or_hovered(true).cloned().collect()
		};

		if opt.force {
			return self.remove_do(opt, tasks);
		}

		tokio::spawn(async move {
			let result = ConfirmProxy::show(if opt.permanently {
				ConfirmCfg::delete(&opt.targets)
			} else {
				ConfirmCfg::trash(&opt.targets)
			});

			if result.await {
				ManagerProxy::remove_do(opt.targets, opt.permanently);
			}
		});
	}

	pub fn remove_do(&mut self, opt: impl Into<Opt>, tasks: &Tasks) {
		let opt = opt.into() as Opt;

		self.tabs.iter_mut().for_each(|t| {
			t.selected.remove_many(&opt.targets, false);
		});

		for u in &opt.targets {
			self.yanked.remove(u);
		}

		self.yanked.catchup_revision(false);
		tasks.file_remove(opt.targets, opt.permanently);
	}
}
