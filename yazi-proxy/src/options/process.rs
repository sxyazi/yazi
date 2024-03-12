use std::ffi::OsString;

use tokio::sync::oneshot;
use yazi_config::open::Opener;
use yazi_shared::event::Cmd;

// --- Exec
pub struct ProcessExecOpt {
	pub opener: Opener,
	pub args:   Vec<OsString>,
	pub done:   oneshot::Sender<()>,
}

impl TryFrom<Cmd> for ProcessExecOpt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> { c.take_data().ok_or(()) }
}
