use yazi_shared::event::CmdCow;

use crate::manager::Manager;

impl Manager {
	pub fn suspend(&mut self, _: CmdCow) {
		#[cfg(unix)]
		unsafe {
			libc::raise(libc::SIGTSTP);
		}
	}
}
