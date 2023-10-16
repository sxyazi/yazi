use crate::manager::Manager;

impl Manager {
	pub fn suspend(&mut self) -> bool {
		#[cfg(unix)]
		tokio::spawn(async move {
			crate::emit!(Stop(true)).await;
			unsafe { libc::raise(libc::SIGTSTP) };
		});
		false
	}
}
