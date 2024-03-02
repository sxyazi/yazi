use yazi_proxy::AppProxy;
use yazi_shared::event::Cmd;

use crate::manager::Manager;

impl Manager {
	pub fn suspend(&mut self, _: Cmd) {
		#[cfg(unix)]
		tokio::spawn(async move {
			AppProxy::stop().await;
			unsafe { libc::raise(libc::SIGTSTP) };
		});
	}
}
