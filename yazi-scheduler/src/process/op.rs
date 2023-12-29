use std::{ffi::OsString, mem};

use tokio::sync::oneshot;
use yazi_plugin::external::ShellOpt;

#[derive(Debug)]
pub struct ProcessOpOpen {
	pub id:     usize,
	pub cmd:    OsString,
	pub args:   Vec<OsString>,
	pub block:  bool,
	pub orphan: bool,
	pub cancel: oneshot::Sender<()>,
}

impl From<&mut ProcessOpOpen> for ShellOpt {
	fn from(op: &mut ProcessOpOpen) -> Self {
		Self {
			cmd:    mem::take(&mut op.cmd),
			args:   mem::take(&mut op.args),
			piped:  false,
			orphan: op.orphan,
		}
	}
}
