use crate::{emit, manager::Manager};

impl Manager {
	pub fn suspend(&mut self) -> bool {
		#[cfg(not(target_os = "windows"))]
		tokio::spawn(async move {
			emit!(Stop(true)).await;
			unsafe { libc::raise(libc::SIGTSTP) };
		});
		false
	}
}
