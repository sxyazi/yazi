use std::ffi::OsString;

use tokio_util::sync::CancellationToken;

use super::ShellOpt;

#[derive(Debug)]
pub struct ProcessOpBlock {
	pub id:   usize,
	pub cmd:  OsString,
	pub args: Vec<OsString>,
}

impl From<ProcessOpBlock> for ShellOpt {
	fn from(op: ProcessOpBlock) -> Self {
		Self { cmd: op.cmd, args: op.args, piped: false, orphan: false }
	}
}

#[derive(Debug)]
pub struct ProcessOpOrphan {
	pub id:   usize,
	pub cmd:  OsString,
	pub args: Vec<OsString>,
}

impl From<ProcessOpOrphan> for ShellOpt {
	fn from(op: ProcessOpOrphan) -> Self {
		Self { cmd: op.cmd, args: op.args, piped: false, orphan: true }
	}
}

#[derive(Debug)]
pub struct ProcessOpBg {
	pub id:   usize,
	pub cmd:  OsString,
	pub args: Vec<OsString>,
	pub ct:   CancellationToken,
}

impl From<ProcessOpBg> for ShellOpt {
	fn from(op: ProcessOpBg) -> Self {
		Self { cmd: op.cmd, args: op.args, piped: true, orphan: false }
	}
}
