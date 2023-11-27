use yazi_shared::event::Exec;

use crate::{manager::Manager, tasks::Tasks};

pub struct Opt {
	force: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { force: e.named.contains_key("force") } }
}

impl Manager {
	pub fn paste(&mut self, opt: impl Into<Opt>, tasks: &Tasks) -> bool {
		let dest = self.cwd();
		let (cut, ref src) = self.yanked;

		let opt = opt.into() as Opt;
		if cut { tasks.file_cut(src, dest, opt.force) } else { tasks.file_copy(src, dest, opt.force) }
	}
}
