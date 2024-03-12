use std::ffi::OsString;

use tokio::sync::oneshot;

use super::ShellOpt;

#[derive(Debug)]
pub struct ProcessOpOpen {
	pub id:     usize,
	pub cmd:    OsString,
	pub args:   Vec<OsString>,
	pub block:  bool,
	pub orphan: bool,
	pub cancel: oneshot::Sender<()>,
}

impl From<ProcessOpOpen> for ShellOpt {
	fn from(op: ProcessOpOpen) -> Self {
		Self { cmd: op.cmd, args: op.args, piped: false, orphan: op.orphan }
	}
}
