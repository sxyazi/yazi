use yazi_shared::event::Cmd;

use crate::manager::Manager;

impl Manager {
	pub fn suspend(&mut self, _: Cmd) {
		#[cfg(unix)]
		unsafe {
			libc::raise(libc::SIGTSTP);
		}
	}
}
