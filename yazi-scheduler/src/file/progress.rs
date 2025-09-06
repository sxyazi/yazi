use serde::Serialize;
use yazi_parser::app::TaskSummary;

// --- Paste
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgPaste {
	pub total_files:     u32,
	pub success_files:   u32,
	pub failed_files:    u32,
	pub total_bytes:     u64,
	pub processed_bytes: u64,
	pub collected:       bool,
	pub cleaned:         bool,
}

impl From<FileProgPaste> for TaskSummary {
	fn from(value: FileProgPaste) -> Self {
		Self {
			total:   value.total_files,
			success: value.success_files,
			failed:  value.failed_files,
			percent: value.percent().map(Into::into),
		}
	}
}

impl FileProgPaste {
	pub fn running(self) -> bool {
		!self.collected || self.success_files + self.failed_files != self.total_files
	}

	pub fn success(self) -> bool { self.collected && self.success_files == self.total_files }

	pub fn cleaned(self) -> bool { self.cleaned }

	pub fn percent(self) -> Option<f32> {
		Some(if self.success() {
			100.0
		} else if self.total_bytes == 0 {
			99.99
		} else {
			99.99f32.min(self.processed_bytes as f32 / self.total_bytes as f32 * 100.0)
		})
	}
}

// --- Link
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgLink {
	pub state: Option<bool>,
}

impl From<FileProgLink> for TaskSummary {
	fn from(value: FileProgLink) -> Self {
		Self {
			total:   1,
			success: (value.state == Some(true)) as u32,
			failed:  (value.state == Some(false)) as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl FileProgLink {
	pub fn running(self) -> bool { self.state.is_none() }

	pub fn success(self) -> bool { self.state == Some(true) }

	pub fn cleaned(self) -> bool { false }

	pub fn percent(self) -> Option<f32> { None }
}

// --- Hardlink
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgHardlink {
	pub total:     u32,
	pub success:   u32,
	pub failed:    u32,
	pub collected: bool,
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

impl FileProgHardlink {
	pub fn running(self) -> bool { !self.collected || self.success + self.failed != self.total }

	pub fn success(self) -> bool { self.collected && self.success == self.total }

	pub fn cleaned(self) -> bool { false }

	pub fn percent(self) -> Option<f32> { None }
}

// --- Delete
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgDelete {
	pub total_files:     u32,
	pub success_files:   u32,
	pub failed_files:    u32,
	pub total_bytes:     u64,
	pub processed_bytes: u64,
	pub collected:       bool,
	pub cleaned:         bool,
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

impl FileProgDelete {
	pub fn running(self) -> bool {
		!self.collected || self.success_files + self.failed_files != self.total_files
	}

	pub fn success(self) -> bool { self.collected && self.success_files == self.total_files }

	pub fn cleaned(self) -> bool { self.cleaned }

	pub fn percent(self) -> Option<f32> {
		Some(if self.success() {
			100.0
		} else if self.total_bytes == 0 {
			99.99
		} else {
			99.99f32.min(self.processed_bytes as f32 / self.total_bytes as f32 * 100.0)
		})
	}
}

// --- Trash
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgTrash {
	pub state: Option<bool>,
}

impl From<FileProgTrash> for TaskSummary {
	fn from(value: FileProgTrash) -> Self {
		Self {
			total:   1,
			success: (value.state == Some(true)) as u32,
			failed:  (value.state == Some(false)) as u32,
			percent: value.percent().map(Into::into),
		}
	}
}

impl FileProgTrash {
	pub fn running(self) -> bool { self.state.is_none() }

	pub fn success(self) -> bool { self.state == Some(true) }

	pub fn cleaned(self) -> bool { false }

	pub fn percent(self) -> Option<f32> { None }
}
