use yazi_config::popup::InputCfg;
use yazi_proxy::{InputProxy, ManagerProxy};
use yazi_shared::{event::Cmd, fs::Url};

use crate::{manager::Manager, tasks::Tasks};

pub struct Opt {
	force:       bool,
	permanently: bool,
	targets:     Vec<Url>,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			force:       c.named.contains_key("force"),
			permanently: c.named.contains_key("permanently"),
			targets:     c.take_data().unwrap_or_default(),
		}
	}
}

impl Manager {
	pub fn remove(&mut self, opt: impl Into<Opt>, tasks: &Tasks) {
		if !self.active_mut().try_escape_visual() {
			return;
		}

		let mut opt = opt.into() as Opt;
		opt.targets = self.selected_or_hovered().into_iter().cloned().collect();

		if opt.force {
			return self.remove_do(opt, tasks);
		}

		tokio::spawn(async move {
			let mut result = InputProxy::show(if opt.permanently {
				InputCfg::delete(opt.targets.len())
			} else {
				InputCfg::trash(opt.targets.len())
			});

			if let Some(Ok(choice)) = result.recv().await {
				if choice != "y" && choice != "Y" {
					return;
				}

				ManagerProxy::remove_do(opt.targets, opt.permanently);
			}
		});
	}

	pub fn remove_do(&mut self, opt: impl Into<Opt>, tasks: &Tasks) {
		let opt = opt.into() as Opt;
		for u in &opt.targets {
			self.active_mut().selected.remove(u);
		}

		tasks.file_remove(opt.targets, opt.permanently);
	}
}
