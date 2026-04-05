use serde::Serialize;

use crate::{CleanupState, Progress, TaskSummary};

// --- Copy
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgCopy {
	pub total_files:     u32,
	pub success_files:   u32,
	pub failed_files:    u32,
	pub total_bytes:     u64,
	pub processed_bytes: u64,
	pub collected:       Option<bool>,
	pub cleaned:         CleanupState,
}

impl From<FileProgCopy> for TaskSummary {
	fn from(value: FileProgCopy) -> Self {
		Self {
			total:   value.total_files,
			success: value.success_files,
			failed:  value.failed_files,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for FileProgCopy {
	fn running(self) -> bool {
		self.cooking_or_cleaning(
			self.collected.is_none() || self.success_files + self.failed_files != self.total_files,
		)
	}

	fn cooked(self) -> bool { self.collected == Some(true) && self.success_files == self.total_files }

	fn failed(self) -> bool { self.cleaned.is_failed() || self.collected == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }

	fn percent(self) -> Option<f32> {
		Some(self.byte_percent(self.processed_bytes, self.total_bytes))
	}
}

// --- Cut
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgCut {
	pub total_files:     u32,
	pub success_files:   u32,
	pub failed_files:    u32,
	pub total_bytes:     u64,
	pub processed_bytes: u64,
	pub collected:       Option<bool>,
	pub cleaned:         CleanupState,
}

impl From<FileProgCut> for TaskSummary {
	fn from(value: FileProgCut) -> Self {
		Self {
			total:   value.total_files,
			success: value.success_files,
			failed:  value.failed_files,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for FileProgCut {
	fn running(self) -> bool {
		self.cooking_or_cleaning(
			self.collected.is_none() || self.success_files + self.failed_files != self.total_files,
		)
	}

	fn cooked(self) -> bool { self.collected == Some(true) && self.success_files == self.total_files }

	fn failed(self) -> bool { self.cleaned.is_failed() || self.collected == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }

	fn percent(self) -> Option<f32> {
		Some(self.byte_percent(self.processed_bytes, self.total_bytes))
	}
}

// --- Link
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgLink {
	pub state:   Option<bool>,
	pub cleaned: CleanupState,
}

impl From<FileProgLink> for TaskSummary {
	fn from(value: FileProgLink) -> Self {
		Self {
			total:   1,
			success: value.success() as u32,
			failed:  value.failed() as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for FileProgLink {
	fn running(self) -> bool { self.cooking_or_cleaning(self.state.is_none()) }

	fn cooked(self) -> bool { self.state == Some(true) }

	fn failed(self) -> bool { self.cleaned.is_failed() || self.state == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }
}

// --- Hardlink
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgHardlink {
	pub total:     u32,
	pub success:   u32,
	pub failed:    u32,
	pub collected: Option<bool>,
	pub cleaned:   CleanupState,
}

impl From<FileProgHardlink> for TaskSummary {
	fn from(value: FileProgHardlink) -> Self {
		Self {
			total:   value.total,
			success: value.success,
			failed:  value.failed,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for FileProgHardlink {
	fn running(self) -> bool {
		self.cooking_or_cleaning(self.collected.is_none() || self.success + self.failed != self.total)
	}

	fn cooked(self) -> bool { self.collected == Some(true) && self.success == self.total }

	fn failed(self) -> bool { self.cleaned.is_failed() || self.collected == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }
}

// --- Delete
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgDelete {
	pub total_files:     u32,
	pub success_files:   u32,
	pub failed_files:    u32,
	pub total_bytes:     u64,
	pub processed_bytes: u64,
	pub collected:       Option<bool>,
	pub cleaned:         CleanupState,
}

impl From<FileProgDelete> for TaskSummary {
	fn from(value: FileProgDelete) -> Self {
		Self {
			total:   value.total_files,
			success: value.success_files,
			failed:  value.failed_files,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for FileProgDelete {
	fn running(self) -> bool {
		self.cooking_or_cleaning(
			self.collected.is_none() || self.success_files + self.failed_files != self.total_files,
		)
	}

	fn cooked(self) -> bool { self.collected == Some(true) && self.success_files == self.total_files }

	fn failed(self) -> bool { self.cleaned.is_failed() || self.collected == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }

	fn percent(self) -> Option<f32> {
		Some(self.byte_percent(self.processed_bytes, self.total_bytes))
	}
}

// --- Trash
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgTrash {
	pub state:   Option<bool>,
	pub cleaned: CleanupState,
}

impl From<FileProgTrash> for TaskSummary {
	fn from(value: FileProgTrash) -> Self {
		Self {
			total:   1,
			success: value.success() as u32,
			failed:  value.failed() as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for FileProgTrash {
	fn running(self) -> bool { self.cooking_or_cleaning(self.state.is_none()) }

	fn cooked(self) -> bool { self.state == Some(true) }

	fn failed(self) -> bool { self.cleaned.is_failed() || self.state == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }
}

// --- Download
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgDownload {
	pub total_files:     u32,
	pub success_files:   u32,
	pub failed_files:    u32,
	pub total_bytes:     u64,
	pub processed_bytes: u64,
	pub collected:       Option<bool>,
	pub cleaned:         CleanupState,
}

impl From<FileProgDownload> for TaskSummary {
	fn from(value: FileProgDownload) -> Self {
		Self {
			total:   value.total_files,
			success: value.success_files,
			failed:  value.failed_files,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for FileProgDownload {
	fn running(self) -> bool {
		self.cooking_or_cleaning(
			self.collected.is_none() || self.success_files + self.failed_files != self.total_files,
		)
	}

	fn cooked(self) -> bool { self.collected == Some(true) && self.success_files == self.total_files }

	fn failed(self) -> bool { self.cleaned.is_failed() || self.collected == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }

	fn percent(self) -> Option<f32> {
		Some(self.byte_percent(self.processed_bytes, self.total_bytes))
	}
}

// --- Upload
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgUpload {
	pub total_files:     u32,
	pub success_files:   u32,
	pub failed_files:    u32,
	pub total_bytes:     u64,
	pub processed_bytes: u64,
	pub collected:       Option<bool>,
	pub cleaned:         CleanupState,
}

impl From<FileProgUpload> for TaskSummary {
	fn from(value: FileProgUpload) -> Self {
		Self {
			total:   value.total_files,
			success: value.success_files,
			failed:  value.failed_files,
			percent: value.percent().map(Into::into),
		}
	}
}

impl Progress for FileProgUpload {
	fn running(self) -> bool {
		self.cooking_or_cleaning(
			self.collected.is_none() || self.success_files + self.failed_files != self.total_files,
		)
	}

	fn cooked(self) -> bool { self.collected == Some(true) && self.success_files == self.total_files }

	fn failed(self) -> bool { self.cleaned.is_failed() || self.collected == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }

	fn percent(self) -> Option<f32> {
		Some(self.byte_percent(self.processed_bytes, self.total_bytes))
	}
}
