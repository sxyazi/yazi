use yazi_scheduler::Scheduler;
use yazi_shared::event::Cmd;

use crate::manager::Manager;

impl Manager {
	pub fn suspend(&mut self, _: Cmd) {
		#[cfg(unix)]
		tokio::spawn(async move {
			Scheduler::app_stop().await;
			unsafe { libc::raise(libc::SIGTSTP) };
		});
	}
}
