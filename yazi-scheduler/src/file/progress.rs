use std::ops::Deref;

use serde::Serialize;
use yazi_fs::remover::Delta;

use crate::{CleanupState, Progress, TaskSummary};

// --- Copy
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgCopy {
	pub(crate) total_files:     u64,
	pub(crate) succeeded_files: u64,
	pub(crate) failed_files:    u64,
	pub(crate) total_bytes:     u64,
	pub(crate) processed_bytes: u64,
	pub(crate) collected:       Option<bool>,
	pub(crate) cleaned:         CleanupState,
}

impl From<FileProgCopy> for TaskSummary {
	fn from(value: FileProgCopy) -> Self {
		Self {
			total_tasks:     1,
			succeeded_tasks: value.success() as u32,
			failed_tasks:    value.failed() as u32,
			total_items:     value.total_files,
			succeeded_items: value.succeeded_files,
			failed_items:    value.failed_files,
			percent:         value.percent().map(Into::into),
		}
	}
}

impl Progress for FileProgCopy {
	fn running(self) -> bool {
		self.cooking_or_cleaning(
			self.collected.is_none() || self.succeeded_files + self.failed_files != self.total_files,
		)
	}

	fn cooked(self) -> bool {
		self.collected == Some(true) && self.succeeded_files == self.total_files
	}

	fn failed(self) -> bool { self.cleaned.is_failed() || self.collected == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }

	fn percent(self) -> Option<f32> {
		Some(self.work_percent(self.processed_bytes, self.total_bytes, self.total_files))
	}
}

// --- Move
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgMove {
	pub(crate) total_files:     u64,
	pub(crate) succeeded_files: u64,
	pub(crate) failed_files:    u64,
	pub(crate) total_bytes:     u64,
	pub(crate) processed_bytes: u64,
	pub(crate) collected:       Option<bool>,
	pub(crate) cleaned:         CleanupState,
}

impl From<FileProgMove> for TaskSummary {
	fn from(value: FileProgMove) -> Self {
		Self {
			total_tasks:     1,
			succeeded_tasks: value.success() as u32,
			failed_tasks:    value.failed() as u32,
			total_items:     value.total_files,
			succeeded_items: value.succeeded_files,
			failed_items:    value.failed_files,
			percent:         value.percent().map(Into::into),
		}
	}
}

impl Progress for FileProgMove {
	fn running(self) -> bool {
		self.cooking_or_cleaning(
			self.collected.is_none() || self.succeeded_files + self.failed_files != self.total_files,
		)
	}

	fn cooked(self) -> bool {
		self.collected == Some(true) && self.succeeded_files == self.total_files
	}

	fn failed(self) -> bool { self.cleaned.is_failed() || self.collected == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }

	fn percent(self) -> Option<f32> {
		Some(self.work_percent(self.processed_bytes, self.total_bytes, self.total_files))
	}
}

// --- Link
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgLink {
	pub(crate) state:   Option<bool>,
	pub(crate) cleaned: CleanupState,
}

impl From<FileProgLink> for TaskSummary {
	fn from(value: FileProgLink) -> Self {
		Self {
			total_tasks:     1,
			succeeded_tasks: value.success() as u32,
			failed_tasks:    value.failed() as u32,
			total_items:     1,
			succeeded_items: value.cooked() as u64,
			failed_items:    (value.state == Some(false)) as u64,
			percent:         value.percent().map(Into::into),
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
	pub(crate) total:     u64,
	pub(crate) succeeded: u64,
	pub(crate) failed:    u64,
	pub(crate) collected: Option<bool>,
	pub(crate) cleaned:   CleanupState,
}

impl From<FileProgHardlink> for TaskSummary {
	fn from(value: FileProgHardlink) -> Self {
		Self {
			total_tasks:     1,
			succeeded_tasks: value.success() as u32,
			failed_tasks:    value.failed() as u32,
			total_items:     value.total,
			succeeded_items: value.succeeded,
			failed_items:    value.failed,
			percent:         value.percent().map(Into::into),
		}
	}
}

impl Progress for FileProgHardlink {
	fn running(self) -> bool {
		self.cooking_or_cleaning(self.collected.is_none() || self.succeeded + self.failed != self.total)
	}

	fn cooked(self) -> bool { self.collected == Some(true) && self.succeeded == self.total }

	fn failed(self) -> bool { self.cleaned.is_failed() || self.collected == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }
}

// --- Delete
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgDelete {
	#[serde(flatten)]
	pub(crate) delta:   Delta,
	pub(crate) state:   Option<bool>,
	pub(crate) cleaned: CleanupState,
}

impl Deref for FileProgDelete {
	type Target = Delta;

	fn deref(&self) -> &Self::Target { &self.delta }
}

impl From<FileProgDelete> for TaskSummary {
	fn from(value: FileProgDelete) -> Self {
		Self {
			total_tasks:     1,
			succeeded_tasks: value.success() as u32,
			failed_tasks:    value.failed() as u32,
			total_items:     value.total,
			succeeded_items: value.succeeded,
			failed_items:    value.failed,
			percent:         value.percent().map(Into::into),
		}
	}
}

impl Progress for FileProgDelete {
	fn running(self) -> bool { self.cooking_or_cleaning(self.state.is_none()) }

	fn cooked(self) -> bool { self.state == Some(true) && self.failed == 0 }

	fn failed(self) -> bool { self.state == Some(false) || self.failed != 0 }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }

	fn percent(self) -> Option<f32> {
		Some(self.work_percent(self.processed, self.workload?, self.total))
	}
}

// --- Trash
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgTrash {
	pub(crate) state:   Option<bool>,
	pub(crate) cleaned: CleanupState,
}

impl From<FileProgTrash> for TaskSummary {
	fn from(value: FileProgTrash) -> Self {
		Self {
			total_tasks:     1,
			succeeded_tasks: value.success() as u32,
			failed_tasks:    value.failed() as u32,
			total_items:     1,
			succeeded_items: value.cooked() as u64,
			failed_items:    (value.state == Some(false)) as u64,
			percent:         value.percent().map(Into::into),
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
	pub(crate) total_files:     u64,
	pub(crate) succeeded_files: u64,
	pub(crate) failed_files:    u64,
	pub(crate) total_bytes:     u64,
	pub(crate) processed_bytes: u64,
	pub(crate) collected:       Option<bool>,
	pub(crate) cleaned:         CleanupState,
}

impl From<FileProgDownload> for TaskSummary {
	fn from(value: FileProgDownload) -> Self {
		Self {
			total_tasks:     1,
			succeeded_tasks: value.success() as u32,
			failed_tasks:    value.failed() as u32,
			total_items:     value.total_files,
			succeeded_items: value.succeeded_files,
			failed_items:    value.failed_files,
			percent:         value.percent().map(Into::into),
		}
	}
}

impl Progress for FileProgDownload {
	fn running(self) -> bool {
		self.cooking_or_cleaning(
			self.collected.is_none() || self.succeeded_files + self.failed_files != self.total_files,
		)
	}

	fn cooked(self) -> bool {
		self.collected == Some(true) && self.succeeded_files == self.total_files
	}

	fn failed(self) -> bool { self.cleaned.is_failed() || self.collected == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }

	fn percent(self) -> Option<f32> {
		Some(self.work_percent(self.processed_bytes, self.total_bytes, self.total_files))
	}
}

// --- Upload
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgUpload {
	pub(crate) total_files:     u64,
	pub(crate) succeeded_files: u64,
	pub(crate) failed_files:    u64,
	pub(crate) total_bytes:     u64,
	pub(crate) processed_bytes: u64,
	pub(crate) collected:       Option<bool>,
	pub(crate) cleaned:         CleanupState,
}

impl From<FileProgUpload> for TaskSummary {
	fn from(value: FileProgUpload) -> Self {
		Self {
			total_tasks:     1,
			succeeded_tasks: value.success() as u32,
			failed_tasks:    value.failed() as u32,
			total_items:     value.total_files,
			succeeded_items: value.succeeded_files,
			failed_items:    value.failed_files,
			percent:         value.percent().map(Into::into),
		}
	}
}

impl Progress for FileProgUpload {
	fn running(self) -> bool {
		self.cooking_or_cleaning(
			self.collected.is_none() || self.succeeded_files + self.failed_files != self.total_files,
		)
	}

	fn cooked(self) -> bool {
		self.collected == Some(true) && self.succeeded_files == self.total_files
	}

	fn failed(self) -> bool { self.cleaned.is_failed() || self.collected == Some(false) }

	fn cleaned(self) -> Option<CleanupState> { Some(self.cleaned) }

	fn percent(self) -> Option<f32> {
		Some(self.work_percent(self.processed_bytes, self.total_bytes, self.total_files))
	}
}
