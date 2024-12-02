use std::collections::HashSet;

use tracing::debug;
use yazi_shared::url::Url;

use super::Tasks;

impl Tasks {
	pub fn file_cut(&self, src: &[&Url], dest: &Url, force: bool) {
		for &u in src {
			let to = dest.join(u.file_name().unwrap());
			if force && *u == to {
				debug!("file_cut: same file, skipping {:?}", to);
			} else {
				self.scheduler.file_cut(u.clone(), to, force);
			}
		}
	}

	pub fn file_copy(&self, src: &[&Url], dest: &Url, force: bool, follow: bool) {
		for &u in src {
			let to = dest.join(u.file_name().unwrap());
			if force && *u == to {
				debug!("file_copy: same file, skipping {:?}", to);
			} else {
				self.scheduler.file_copy(u.clone(), to, force, follow);
			}
		}
	}

	pub fn file_link(&self, src: &HashSet<Url>, dest: &Url, relative: bool, force: bool) {
		for u in src {
			let to = dest.join(u.file_name().unwrap());
			if force && *u == to {
				debug!("file_link: same file, skipping {:?}", to);
			} else {
				self.scheduler.file_link(u.clone(), to, relative, force);
			}
		}
	}

	pub fn file_hardlink(&self, src: &HashSet<Url>, dest: &Url, force: bool, follow: bool) {
		for u in src {
			let to = dest.join(u.file_name().unwrap());
			if force && *u == to {
				debug!("file_hardlink: same file, skipping {:?}", to);
			} else {
				self.scheduler.file_hardlink(u.clone(), to, force, follow);
			}
		}
	}

	pub fn file_remove(&self, targets: Vec<Url>, permanently: bool) {
		for u in targets {
			if permanently {
				self.scheduler.file_delete(u);
			} else {
				self.scheduler.file_trash(u);
			}
		}
	}
}
