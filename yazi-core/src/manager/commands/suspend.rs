use yazi_shared::event::Exec;

use crate::{manager::Manager, Ctx};

pub struct Opt;
impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Manager {
	pub fn suspend(&mut self, _: impl Into<Opt>) -> bool {
		#[cfg(unix)]
		tokio::spawn(async move {
			Ctx::stop().await;
			unsafe { libc::raise(libc::SIGTSTP) };
		});
		false
	}
}
