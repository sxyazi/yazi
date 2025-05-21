use std::ffi::OsString;

use tokio::sync::mpsc;
use yazi_shared::url::Url;

use super::ShellOpt;

// --- Block
#[derive(Debug)]
pub struct ProcessInBlock {
	pub id:   usize,
	pub cwd:  Url,
	pub cmd:  OsString,
	pub args: Vec<OsString>,
}

impl From<ProcessInBlock> for ShellOpt {
	fn from(r#in: ProcessInBlock) -> Self {
		Self { cwd: r#in.cwd, cmd: r#in.cmd, args: r#in.args, piped: false, orphan: false }
	}
}

// --- Orphan
#[derive(Debug)]
pub struct ProcessInOrphan {
	pub id:   usize,
	pub cwd:  Url,
	pub cmd:  OsString,
	pub args: Vec<OsString>,
}

impl From<ProcessInOrphan> for ShellOpt {
	fn from(r#in: ProcessInOrphan) -> Self {
		Self { cwd: r#in.cwd, cmd: r#in.cmd, args: r#in.args, piped: false, orphan: true }
	}
}

// --- Bg
#[derive(Debug)]
pub struct ProcessInBg {
	pub id:     usize,
	pub cwd:    Url,
	pub cmd:    OsString,
	pub args:   Vec<OsString>,
	pub cancel: mpsc::Receiver<()>,
}

impl From<ProcessInBg> for ShellOpt {
	fn from(r#in: ProcessInBg) -> Self {
		Self { cwd: r#in.cwd, cmd: r#in.cmd, args: r#in.args, piped: true, orphan: false }
	}
}
