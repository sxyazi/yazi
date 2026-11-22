use yazi_macro::debug;
use yazi_scheduler::file::{FileInCopy, FileInLink, FileInMove};
use yazi_shared::url::{AsUrl, UrlBuf, UrlLike};

use super::Tasks;
use crate::mgr::Yanked;

impl Tasks {
	pub fn file_move(&self, src: &Yanked, dest: impl AsUrl, force: bool) {
		let dest = dest.as_url();
		self.scheduler.behavior.reset();

		for u in src.urls() {
			let Some(Ok(to)) = u.name().map(|n| dest.try_join(n)) else {
				debug!("file_move: cannot join {u} with {dest}");
				continue;
			};
			self.scheduler.file_move(FileInMove::new(u.clone(), to, force));
		}
	}

	pub fn file_copy(&self, src: &Yanked, dest: impl AsUrl, follow: bool, force: bool) {
		let dest = dest.as_url();
		self.scheduler.behavior.reset();

		for u in src.urls() {
			let Some(Ok(to)) = u.name().map(|n| dest.try_join(n)) else {
				debug!("file_copy: cannot join {u} with {dest}");
				continue;
			};
			self.scheduler.file_copy(FileInCopy::new(u.clone(), to, follow, force));
		}
	}

	pub fn file_link(
		&self,
		src: &Yanked,
		dest: impl AsUrl,
		relative: bool,
		follow: bool,
		force: bool,
	) {
		let dest = dest.as_url();
		self.scheduler.behavior.reset();

		for u in src.urls() {
			let Some(Ok(to)) = u.name().map(|n| dest.try_join(n)) else {
				debug!("file_link: cannot join {u} with {dest}");
				continue;
			};
			self.scheduler.file_link(FileInLink::new(u.clone(), to, relative, follow, force));
		}
	}

	pub fn file_hardlink(&self, src: &Yanked, dest: impl AsUrl, follow: bool, force: bool) {
		let dest = dest.as_url();
		self.scheduler.behavior.reset();

		for u in src.urls() {
			let Some(Ok(to)) = u.name().map(|n| dest.try_join(n)) else {
				debug!("file_hardlink: cannot join {u} with {dest}");
				continue;
			};
			self.scheduler.file_hardlink(u.clone(), to, follow, force);
		}
	}

	pub fn file_remove(&self, targets: Vec<UrlBuf>, permanently: bool) {
		for u in targets {
			if permanently {
				self.scheduler.file_delete(u);
			} else {
				self.scheduler.file_trash(u);
			}
		}
	}
}
