use std::ffi::OsString;

use tokio::sync::mpsc;
use yazi_shared::url::Url;

use super::ShellOpt;

// --- Block
#[derive(Debug)]
pub struct ProcessOpBlock {
	pub id:   usize,
	pub cwd:  Url,
	pub cmd:  OsString,
	pub args: Vec<OsString>,
}

impl From<ProcessOpBlock> for ShellOpt {
	fn from(op: ProcessOpBlock) -> Self {
		Self { cwd: op.cwd, cmd: op.cmd, args: op.args, piped: false, orphan: false }
	}
}

// --- Orphan
#[derive(Debug)]
pub struct ProcessOpOrphan {
	pub id:   usize,
	pub cwd:  Url,
	pub cmd:  OsString,
	pub args: Vec<OsString>,
}

impl From<ProcessOpOrphan> for ShellOpt {
	fn from(op: ProcessOpOrphan) -> Self {
		Self { cwd: op.cwd, cmd: op.cmd, args: op.args, piped: false, orphan: true }
	}
}

// --- Bg
#[derive(Debug)]
pub struct ProcessOpBg {
	pub id:     usize,
	pub cwd:    Url,
	pub cmd:    OsString,
	pub args:   Vec<OsString>,
	pub cancel: mpsc::Receiver<()>,
}

impl From<ProcessOpBg> for ShellOpt {
	fn from(op: ProcessOpBg) -> Self {
		Self { cwd: op.cwd, cmd: op.cmd, args: op.args, piped: true, orphan: false }
	}
}
