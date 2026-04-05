use serde::Serialize;

use crate::{CleanupState, TaskSummary, dispatch_progress, fetch::FetchProg, file::{FileProgCopy, FileProgCut, FileProgDelete, FileProgDownload, FileProgHardlink, FileProgLink, FileProgTrash, FileProgUpload}, impl_from_prog, plugin::PluginProgEntry, preload::PreloadProg, process::{ProcessProgBg, ProcessProgBlock, ProcessProgOrphan}, size::SizeProg};

pub trait Progress: Copy {
	// Whether the task is still cooking or cleaning.
	fn running(self) -> bool;

	// Whether the task succeeded, regardless cleanup.
	// For tasks without a cleanup, this is the same as `success()`.
	fn cooked(self) -> bool;

	// Whether the task fully succeeded, including cleanup if applicable.
	fn success(self) -> bool {
		match self.cleaned() {
			None | Some(CleanupState::Success) => self.cooked(),
			Some(CleanupState::Pending | CleanupState::Failed) => false,
		}
	}

	// Whether the task fully failed.
	//
	// For tasks with a collect phase, e.g. gathering files to copy:
	//   collect failed, or cleanup failed if applicable, regardless main work.
	// For tasks without a collect phase:
	//   main work failed, or cleanup failed if applicable.
	fn failed(self) -> bool;

	// Cleanup state if the task has a cleanup phase, otherwise `None`.
	fn cleaned(self) -> Option<CleanupState> { None }

	// Optional percentage for UI display.
	fn percent(self) -> Option<f32> { None }

	// Helper for tasks that are still cooking or cleaning.
	fn cooking_or_cleaning(self, cooking: bool) -> bool {
		cooking || (self.cooked() && self.cleaned() == Some(CleanupState::Pending))
	}

	// Helper for byte-based progress calculations used by file transfer tasks.
	fn byte_percent(self, processed_bytes: u64, total_bytes: u64) -> f32 {
		if self.success() {
			100.0
		} else if self.failed() {
			0.0
		} else if total_bytes != 0 {
			99.99f32.min(processed_bytes as f32 / total_bytes as f32 * 100.0)
		} else {
			99.99
		}
	}
}

// --- TaskProg
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind")]
pub enum TaskProg {
	// File
	FileCopy(FileProgCopy),
	FileCut(FileProgCut),
	FileLink(FileProgLink),
	FileHardlink(FileProgHardlink),
	FileDelete(FileProgDelete),
	FileTrash(FileProgTrash),
	FileDownload(FileProgDownload),
	FileUpload(FileProgUpload),
	// Plugin
	PluginEntry(PluginProgEntry),
	// Fetch
	Fetch(FetchProg),
	// Preload
	Preload(PreloadProg),
	// Size
	Size(SizeProg),
	// Process
	ProcessBlock(ProcessProgBlock),
	ProcessOrphan(ProcessProgOrphan),
	ProcessBg(ProcessProgBg),
}

impl_from_prog! {
	// File
	FileCopy(FileProgCopy), FileCut(FileProgCut), FileLink(FileProgLink), FileHardlink(FileProgHardlink), FileDelete(FileProgDelete), FileTrash(FileProgTrash), FileDownload(FileProgDownload), FileUpload(FileProgUpload),
	// Plugin
	PluginEntry(PluginProgEntry),
	// Fetch
	Fetch(FetchProg),
	// Preload
	Preload(PreloadProg),
	// Size
	Size(SizeProg),
	// Process
	ProcessBlock(ProcessProgBlock), ProcessOrphan(ProcessProgOrphan), ProcessBg(ProcessProgBg),
}

impl From<TaskProg> for TaskSummary {
	fn from(value: TaskProg) -> Self {
		match value {
			// File
			TaskProg::FileCopy(p) => p.into(),
			TaskProg::FileCut(p) => p.into(),
			TaskProg::FileLink(p) => p.into(),
			TaskProg::FileHardlink(p) => p.into(),
			TaskProg::FileDelete(p) => p.into(),
			TaskProg::FileTrash(p) => p.into(),
			TaskProg::FileDownload(p) => p.into(),
			TaskProg::FileUpload(p) => p.into(),
			// Plugin
			TaskProg::PluginEntry(p) => p.into(),
			// Prework
			TaskProg::Fetch(p) => p.into(),
			TaskProg::Preload(p) => p.into(),
			TaskProg::Size(p) => p.into(),
			// Process
			TaskProg::ProcessBlock(p) => p.into(),
			TaskProg::ProcessOrphan(p) => p.into(),
			TaskProg::ProcessBg(p) => p.into(),
		}
	}
}

impl Progress for TaskProg {
	fn running(self) -> bool { dispatch_progress!(self, running) }

	fn cooked(self) -> bool { dispatch_progress!(self, cooked) }

	fn success(self) -> bool { dispatch_progress!(self, success) }

	fn failed(self) -> bool { dispatch_progress!(self, failed) }

	fn cleaned(self) -> Option<CleanupState> { dispatch_progress!(self, cleaned) }

	fn percent(self) -> Option<f32> { dispatch_progress!(self, percent) }
}

impl TaskProg {
	pub(crate) fn is_user(self) -> bool {
		match self {
			// File
			Self::FileCopy(_) => true,
			Self::FileCut(_) => true,
			Self::FileLink(_) => true,
			Self::FileHardlink(_) => true,
			Self::FileDelete(_) => true,
			Self::FileTrash(_) => true,
			Self::FileDownload(_) => true,
			Self::FileUpload(_) => true,
			// Plugin
			Self::PluginEntry(_) => true,
			// Prework
			Self::Fetch(_) => false,
			Self::Preload(_) => false,
			Self::Size(_) => false,
			// Process
			Self::ProcessBlock(_) => true,
			Self::ProcessOrphan(_) => true,
			Self::ProcessBg(_) => true,
		}
	}
}
