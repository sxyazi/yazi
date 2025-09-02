use std::ffi::OsString;

use tokio::sync::mpsc;
use yazi_shared::{Id, url::UrlBuf};

use super::ShellOpt;

// --- Block
#[derive(Debug)]
pub(crate) struct ProcessInBlock {
	pub(crate) id:   Id,
	pub(crate) cwd:  UrlBuf,
	pub(crate) cmd:  OsString,
	pub(crate) args: Vec<OsString>,
}

impl From<ProcessInBlock> for ShellOpt {
	fn from(r#in: ProcessInBlock) -> Self {
		Self { cwd: r#in.cwd, cmd: r#in.cmd, args: r#in.args, piped: false, orphan: false }
	}
}

// --- Orphan
#[derive(Debug)]
pub(crate) struct ProcessInOrphan {
	pub(crate) id:   Id,
	pub(crate) cwd:  UrlBuf,
	pub(crate) cmd:  OsString,
	pub(crate) args: Vec<OsString>,
}

impl From<ProcessInOrphan> for ShellOpt {
	fn from(r#in: ProcessInOrphan) -> Self {
		Self { cwd: r#in.cwd, cmd: r#in.cmd, args: r#in.args, piped: false, orphan: true }
	}
}

// --- Bg
#[derive(Debug)]
pub(crate) struct ProcessInBg {
	pub(crate) id:     Id,
	pub(crate) cwd:    UrlBuf,
	pub(crate) cmd:    OsString,
	pub(crate) args:   Vec<OsString>,
	pub(crate) cancel: mpsc::Receiver<()>,
}

impl From<ProcessInBg> for ShellOpt {
	fn from(r#in: ProcessInBg) -> Self {
		Self { cwd: r#in.cwd, cmd: r#in.cmd, args: r#in.args, piped: true, orphan: false }
	}
}
