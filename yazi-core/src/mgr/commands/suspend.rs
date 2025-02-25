use yazi_shared::event::CmdCow;

use crate::mgr::Mgr;

impl Mgr {
	pub fn suspend(&mut self, _: CmdCow) {
		#[cfg(unix)]
		unsafe {
			libc::raise(libc::SIGTSTP);
		}
	}
}
