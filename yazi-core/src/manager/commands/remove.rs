use yazi_config::popup::ConfirmCfg;
use yazi_proxy::{ConfirmProxy, ManagerProxy};
use yazi_shared::{event::CmdCow, url::Url};

use crate::{manager::Manager, tasks::Tasks};

struct Opt {
	force:       bool,
	permanently: bool,
	hovered:     bool,
	targets:     Vec<Url>,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			force:       c.bool("force"),
			permanently: c.bool("permanently"),
			hovered:     c.bool("hovered"),
			targets:     c.take_any("targets").unwrap_or_default(),
		}
	}
}

impl Manager {
	#[yazi_codegen::command]
	pub fn remove(&mut self, mut opt: Opt, tasks: &Tasks) {
		if !self.active_mut().try_escape_visual() {
			return;
		}

		opt.targets = if opt.hovered {
			self.hovered().map_or(vec![], |h| vec![h.url.clone()])
		} else {
			self.selected_or_hovered(true).cloned().collect()
		};

		if opt.targets.is_empty() {
			return;
		} else if opt.force {
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

	#[yazi_codegen::command]
	pub fn remove_do(&mut self, opt: Opt, tasks: &Tasks) {
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
