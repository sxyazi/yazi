use std::env;

use yazi_config::keymap::{Exec, KeymapLayer};

use crate::{emit, manager::Manager};

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Manager {
	#[inline]
	pub fn _refresh() {
		emit!(Call(Exec::call("refresh", vec![]).vec(), KeymapLayer::Manager));
	}

	pub fn refresh(&mut self, _: impl Into<Opt>) -> bool {
		env::set_current_dir(self.cwd()).ok();
		env::set_var("PWD", self.cwd());

		self.active_mut().apply_files_attrs(false);

		if let Some(f) = self.parent() {
			self.watcher.trigger_dirs(&[self.cwd(), &f.cwd]);
		} else {
			self.watcher.trigger_dirs(&[self.cwd()]);
		}

		Self::_hover(None);
		false
	}
}
