use yazi_config::popup::ConfirmCfg;
use yazi_parser::mgr::RemoveOpt;
use yazi_proxy::{ConfirmProxy, MgrProxy};

use crate::{mgr::Mgr, tasks::Tasks};

impl Mgr {
	#[yazi_codegen::command]
	pub fn remove(&mut self, mut opt: RemoveOpt, tasks: &Tasks) {
		if !self.active_mut().try_escape_visual() {
			return;
		}

		opt.targets = if opt.hovered {
			self.hovered().map_or(vec![], |h| vec![h.url.clone()])
		} else {
			self.selected_or_hovered().cloned().collect()
		};

		if opt.targets.is_empty() {
			return;
		} else if opt.force {
			return self.remove_do(opt, tasks);
		}

		let confirm = ConfirmProxy::show(if opt.permanently {
			ConfirmCfg::delete(&opt.targets)
		} else {
			ConfirmCfg::trash(&opt.targets)
		});

		tokio::spawn(async move {
			if confirm.await {
				MgrProxy::remove_do(opt.targets, opt.permanently);
			}
		});
	}

	#[yazi_codegen::command]
	pub fn remove_do(&mut self, opt: RemoveOpt, tasks: &Tasks) {
		self.tabs.iter_mut().for_each(|t| {
			t.selected.remove_many(&opt.targets);
		});

		for u in &opt.targets {
			self.yanked.remove(u);
		}

		self.yanked.catchup_revision(false);
		tasks.file_remove(opt.targets, opt.permanently);
	}
}
