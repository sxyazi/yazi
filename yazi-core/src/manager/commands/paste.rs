use yazi_dds::Pubsub;
use yazi_shared::event::Cmd;

use crate::{manager::Manager, tasks::Tasks};

pub struct Opt {
	force: bool,
	follow: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self {
		Self { force: c.named.contains_key("force"), follow: c.named.contains_key("follow") }
	}
}

impl Manager {
	pub fn paste(&mut self, opt: impl Into<Opt>, tasks: &Tasks) {
		let opt = opt.into() as Opt;
		let (src, dest) = (self.yanked.iter().collect::<Vec<_>>(), self.cwd());

		if self.yanked.cut {
			tasks.file_cut(&src, dest, opt.force);

			src.iter().for_each(|source_file| {
				if let Some(name) = source_file.file_name().map(|s| dest.join(s)) {
					Pubsub::pub_from_move(self.tabs.cursor, source_file, &name);
				}
			});

			self.tabs.iter_mut().for_each(|t| _ = t.selected.remove_many(&src, false));
			self.unyank(());
		} else {
			tasks.file_copy(&src, dest, opt.force, opt.follow);
		}
	}
}
