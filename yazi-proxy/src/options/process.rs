use std::ffi::OsString;

use yazi_shared::event::Cmd;

// --- Exec
#[derive(Default)]
pub struct ProcessExecOpt {
	pub cmd:    OsString,
	pub args:   Vec<OsString>,
	pub block:  bool,
	pub orphan: bool,
}

impl TryFrom<Cmd> for ProcessExecOpt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> { c.take_data().ok_or(()) }
}
