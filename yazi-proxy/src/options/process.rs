use std::{borrow::Cow, ffi::OsString};

use tokio::sync::oneshot;
use yazi_config::open::Opener;
use yazi_shared::event::CmdCow;

// --- Exec
pub struct ProcessExecOpt {
	pub args:   Vec<OsString>,
	pub opener: Cow<'static, Opener>,
	pub done:   oneshot::Sender<()>,
}

impl TryFrom<CmdCow> for ProcessExecOpt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> { c.take_any("option").ok_or(()) }
}
