use std::io;

use crate::{Task, TaskProg};

// --- Copy
#[derive(Debug)]
pub(crate) enum FileOutCopy {
	New(u64),
	Deform(String),
	Succ,
	Fail(String),
	Clean,
}

impl From<anyhow::Error> for FileOutCopy {
	fn from(value: anyhow::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl From<std::io::Error> for FileOutCopy {
	fn from(value: std::io::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl FileOutCopy {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FileCopy(prog) = &mut task.prog else { return };
		match self {
			Self::New(bytes) => {
				prog.total_files += 1;
				prog.total_bytes += bytes;
			}
			Self::Deform(reason) => {
				prog.total_files += 1;
				prog.failed_files += 1;
				task.log(reason);
			}
			Self::Succ => {
				prog.collected = Some(true);
			}
			Self::Fail(reason) => {
				prog.collected = Some(false);
				task.log(reason);
			}
			Self::Clean => {
				prog.cleaned = Some(true);
			}
		}
	}
}

// --- CopyDo
#[derive(Debug)]
pub(crate) enum FileOutCopyDo {
	Adv(u64),
	Log(String),
	Succ,
	Fail(String),
}

impl From<anyhow::Error> for FileOutCopyDo {
	fn from(value: anyhow::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl FileOutCopyDo {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FileCopy(prog) = &mut task.prog else { return };
		match self {
			Self::Adv(size) => {
				prog.processed_bytes += size;
			}
			Self::Log(line) => {
				task.log(line);
			}
			Self::Succ => {
				prog.success_files += 1;
			}
			Self::Fail(reason) => {
				prog.failed_files += 1;
				task.log(reason);
			}
		}
	}
}

// --- Cut
#[derive(Debug)]
pub(crate) enum FileOutCut {
	New(u64),
	Deform(String),
	Succ,
	Fail(String),
	Clean(io::Result<()>),
}

impl From<anyhow::Error> for FileOutCut {
	fn from(value: anyhow::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl From<std::io::Error> for FileOutCut {
	fn from(value: std::io::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl FileOutCut {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FileCut(prog) = &mut task.prog else { return };
		match self {
			Self::New(bytes) => {
				prog.total_files += 1;
				prog.total_bytes += bytes;
			}
			Self::Deform(reason) => {
				prog.total_files += 1;
				prog.failed_files += 1;
				task.log(reason);
			}
			Self::Succ => {
				prog.collected = Some(true);
			}
			Self::Fail(reason) => {
				prog.collected = Some(false);
				task.log(reason);
			}
			Self::Clean(Ok(())) => {
				prog.cleaned = Some(true);
			}
			Self::Clean(Err(reason)) => {
				prog.cleaned = Some(false);
				task.log(format!("Failed cleaning up cut file: {reason:?}"));
			}
		}
	}
}

// --- CutDo
#[derive(Debug)]
pub(crate) enum FileOutCutDo {
	Adv(u64),
	Log(String),
	Succ,
	Fail(String),
}

impl From<anyhow::Error> for FileOutCutDo {
	fn from(value: anyhow::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl FileOutCutDo {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FileCut(prog) = &mut task.prog else { return };
		match self {
			Self::Adv(size) => {
				prog.processed_bytes += size;
			}
			Self::Log(line) => {
				task.log(line);
			}
			Self::Succ => {
				prog.success_files += 1;
			}
			Self::Fail(reason) => {
				prog.failed_files += 1;
				task.log(reason);
			}
		}
	}
}

// --- Link
#[derive(Debug)]
pub(crate) enum FileOutLink {
	Succ,
	Fail(String),
}

impl From<anyhow::Error> for FileOutLink {
	fn from(value: anyhow::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl FileOutLink {
	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileLink(prog) = &mut task.prog {
			match self {
				Self::Succ => {
					prog.state = Some(true);
				}
				Self::Fail(reason) => {
					prog.state = Some(false);
					task.log(reason);
				}
			}
		} else if let TaskProg::FileCopy(prog) = &mut task.prog {
			match self {
				Self::Succ => {
					prog.success_files += 1;
				}
				Self::Fail(reason) => {
					prog.failed_files += 1;
					task.log(reason);
				}
			}
		} else if let TaskProg::FileCut(prog) = &mut task.prog {
			match self {
				Self::Succ => {
					prog.success_files += 1;
				}
				Self::Fail(reason) => {
					prog.failed_files += 1;
					task.log(reason);
				}
			}
		}
	}
}

// --- Hardlink
#[derive(Debug)]
pub(crate) enum FileOutHardlink {
	New,
	Deform(String),
	Succ,
	Fail(String),
}

impl From<anyhow::Error> for FileOutHardlink {
	fn from(value: anyhow::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl From<std::io::Error> for FileOutHardlink {
	fn from(value: std::io::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl FileOutHardlink {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FileHardlink(prog) = &mut task.prog else { return };
		match self {
			Self::New => {
				prog.total += 1;
			}
			Self::Deform(reason) => {
				prog.total += 1;
				prog.failed += 1;
				task.log(reason);
			}
			Self::Succ => {
				prog.collected = Some(true);
			}
			Self::Fail(reason) => {
				prog.collected = Some(false);
				task.log(reason);
			}
		}
	}
}

// --- HardlinkDo
#[derive(Debug)]
pub(crate) enum FileOutHardlinkDo {
	Succ,
	Fail(String),
}

impl From<anyhow::Error> for FileOutHardlinkDo {
	fn from(value: anyhow::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl FileOutHardlinkDo {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FileHardlink(prog) = &mut task.prog else { return };
		match self {
			Self::Succ => {
				prog.success += 1;
			}
			Self::Fail(reason) => {
				prog.failed += 1;
				task.log(reason);
			}
		}
	}
}

// --- Delete
#[derive(Debug)]
pub(crate) enum FileOutDelete {
	New(u64),
	Succ,
	Fail(String),
	Clean(io::Result<()>),
}

impl From<anyhow::Error> for FileOutDelete {
	fn from(value: anyhow::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl FileOutDelete {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FileDelete(prog) = &mut task.prog else { return };
		match self {
			Self::New(size) => {
				prog.total_files += 1;
				prog.total_bytes += size;
			}
			Self::Succ => {
				prog.collected = Some(true);
			}
			Self::Fail(reason) => {
				prog.collected = Some(false);
				task.log(reason);
			}
			Self::Clean(Ok(())) => {
				prog.cleaned = Some(true);
			}
			Self::Clean(Err(reason)) => {
				prog.cleaned = Some(false);
				task.log(format!("Failed cleaning up deleted file: {reason:?}"));
			}
		}
	}
}

// --- DeleteDo
#[derive(Debug)]
pub(crate) enum FileOutDeleteDo {
	Succ(u64),
	Fail(String),
}

impl From<anyhow::Error> for FileOutDeleteDo {
	fn from(value: anyhow::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl FileOutDeleteDo {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FileDelete(prog) = &mut task.prog else { return };
		match self {
			Self::Succ(size) => {
				prog.success_files += 1;
				prog.processed_bytes += size;
			}
			Self::Fail(reason) => {
				prog.failed_files += 1;
				task.log(reason);
			}
		}
	}
}

// --- Trash
#[derive(Debug)]
pub(crate) enum FileOutTrash {
	Succ,
	Fail(String),
	Clean,
}

impl From<anyhow::Error> for FileOutTrash {
	fn from(value: anyhow::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl FileOutTrash {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FileTrash(prog) = &mut task.prog else { return };
		match self {
			Self::Succ => {
				prog.state = Some(true);
			}
			Self::Fail(reason) => {
				prog.state = Some(false);
				task.log(reason);
			}
			Self::Clean => {
				prog.cleaned = Some(true);
			}
		}
	}
}

// --- Download
#[derive(Debug)]
pub(crate) enum FileOutDownload {
	New(u64),
	Deform(String),
	Succ,
	Fail(String),
	Clean,
}

impl From<anyhow::Error> for FileOutDownload {
	fn from(value: anyhow::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl FileOutDownload {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FileDownload(prog) = &mut task.prog else { return };
		match self {
			Self::New(bytes) => {
				prog.total_files += 1;
				prog.total_bytes += bytes;
			}
			Self::Deform(reason) => {
				prog.total_files += 1;
				prog.failed_files += 1;
				task.log(reason);
			}
			Self::Succ => {
				prog.collected = Some(true);
			}
			Self::Fail(reason) => {
				prog.collected = Some(false);
				task.log(reason);
			}
			Self::Clean => {
				prog.cleaned = Some(true);
			}
		}
	}
}

// --- DownloadDo
#[derive(Debug)]
pub(crate) enum FileOutDownloadDo {
	Adv(u64),
	Log(String),
	Succ,
	Fail(String),
}

impl From<anyhow::Error> for FileOutDownloadDo {
	fn from(value: anyhow::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl FileOutDownloadDo {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FileDownload(prog) = &mut task.prog else { return };
		match self {
			Self::Adv(size) => {
				prog.processed_bytes += size;
			}
			Self::Log(line) => {
				task.log(line);
			}
			Self::Succ => {
				prog.success_files += 1;
			}
			Self::Fail(reason) => {
				prog.failed_files += 1;
				task.log(reason);
			}
		}
	}
}

// --- Upload
#[derive(Debug)]
pub(crate) enum FileOutUpload {
	New(u64),
	Deform(String),
	Succ,
	Fail(String),
}

impl From<anyhow::Error> for FileOutUpload {
	fn from(value: anyhow::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl FileOutUpload {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FileUpload(prog) = &mut task.prog else { return };
		match self {
			Self::New(bytes) => {
				prog.total_files += 1;
				prog.total_bytes += bytes;
			}
			Self::Deform(reason) => {
				prog.total_files += 1;
				prog.failed_files += 1;
				task.log(reason);
			}
			Self::Succ => {
				prog.collected = Some(true);
			}
			Self::Fail(reason) => {
				prog.collected = Some(false);
				task.log(reason);
			}
		}
	}
}

// --- UploadDo
#[derive(Debug)]
pub(crate) enum FileOutUploadDo {
	Adv(u64),
	Succ,
	Fail(String),
}

impl From<anyhow::Error> for FileOutUploadDo {
	fn from(value: anyhow::Error) -> Self { Self::Fail(format!("{value:?}")) }
}

impl FileOutUploadDo {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FileUpload(prog) = &mut task.prog else { return };
		match self {
			Self::Adv(size) => {
				prog.processed_bytes += size;
			}
			Self::Succ => {
				prog.success_files += 1;
			}
			Self::Fail(reason) => {
				prog.failed_files += 1;
				task.log(reason);
			}
		}
	}
}
