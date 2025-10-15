use crate::{Task, TaskProg};

// --- Paste
#[derive(Debug)]
pub(crate) enum FileOutPaste {
	New(u64),
	Deform(String),
	Succ,
	Fail(String),
	Clean,
}

impl From<std::io::Error> for FileOutPaste {
	fn from(value: std::io::Error) -> Self { Self::Fail(value.to_string()) }
}

impl FileOutPaste {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FilePaste(prog) = &mut task.prog else { return };
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
				prog.cleaned = true;
			}
		}
	}
}

// --- PasteDo
#[derive(Debug)]
pub(crate) enum FileOutPasteDo {
	Adv(u64),
	Log(String),
	Succ,
	Fail(String),
}

impl From<std::io::Error> for FileOutPasteDo {
	fn from(value: std::io::Error) -> Self { Self::Fail(value.to_string()) }
}

impl FileOutPasteDo {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::FilePaste(prog) = &mut task.prog else { return };
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
	fn from(value: anyhow::Error) -> Self { Self::Fail(value.to_string()) }
}

impl From<std::io::Error> for FileOutLink {
	fn from(value: std::io::Error) -> Self { Self::Fail(value.to_string()) }
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
		} else if let TaskProg::FilePaste(prog) = &mut task.prog {
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

impl From<std::io::Error> for FileOutHardlink {
	fn from(value: std::io::Error) -> Self { Self::Fail(value.to_string()) }
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

impl From<std::io::Error> for FileOutHardlinkDo {
	fn from(value: std::io::Error) -> Self { Self::Fail(value.to_string()) }
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
	Clean,
}

impl From<std::io::Error> for FileOutDelete {
	fn from(value: std::io::Error) -> Self { Self::Fail(value.to_string()) }
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
			Self::Clean => {
				prog.cleaned = true;
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

impl From<std::io::Error> for FileOutDeleteDo {
	fn from(value: std::io::Error) -> Self { Self::Fail(value.to_string()) }
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
}

impl From<std::io::Error> for FileOutTrash {
	fn from(value: std::io::Error) -> Self { Self::Fail(value.to_string()) }
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
}

impl From<std::io::Error> for FileOutDownload {
	fn from(value: std::io::Error) -> Self { Self::Fail(value.to_string()) }
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

impl From<std::io::Error> for FileOutDownloadDo {
	fn from(value: std::io::Error) -> Self { Self::Fail(value.to_string()) }
}

impl From<anyhow::Error> for FileOutDownloadDo {
	fn from(value: anyhow::Error) -> Self { Self::Fail(value.to_string()) }
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

impl From<std::io::Error> for FileOutUpload {
	fn from(value: std::io::Error) -> Self { Self::Fail(value.to_string()) }
}

impl From<anyhow::Error> for FileOutUpload {
	fn from(value: anyhow::Error) -> Self { Self::Fail(value.to_string()) }
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

impl From<std::io::Error> for FileOutUploadDo {
	fn from(value: std::io::Error) -> Self { Self::Fail(value.to_string()) }
}

impl From<anyhow::Error> for FileOutUploadDo {
	fn from(value: anyhow::Error) -> Self { Self::Fail(value.to_string()) }
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
