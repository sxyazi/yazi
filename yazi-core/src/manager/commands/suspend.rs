use yazi_config::keymap::Exec;

use crate::manager::Manager;

pub struct Opt;
impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Manager {
	pub fn suspend(&mut self, _: impl Into<Opt>) -> bool {
		#[cfg(unix)]
		tokio::spawn(async move {
			crate::emit!(Stop(true)).await;
			unsafe { libc::raise(libc::SIGTSTP) };
		});
		false
	}
}
