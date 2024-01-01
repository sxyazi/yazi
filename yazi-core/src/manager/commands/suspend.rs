use yazi_scheduler::Scheduler;
use yazi_shared::event::Exec;

use crate::manager::Manager;

impl Manager {
	pub fn suspend(&mut self, _: &Exec) {
		#[cfg(unix)]
		tokio::spawn(async move {
			Scheduler::app_stop().await;
			unsafe { libc::raise(libc::SIGTSTP) };
		});
	}
}
