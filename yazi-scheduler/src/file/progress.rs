use serde::Serialize;
use yazi_parser::app::TaskSummary;

// --- Copy
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgCopy {
	pub total_files:     u32,
	pub success_files:   u32,
	pub failed_files:    u32,
	pub total_bytes:     u64,
	pub processed_bytes: u64,
	pub collected:       Option<bool>,
	pub cleaned:         Option<bool>,
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

impl FileProgCopy {
	pub fn cooked(self) -> bool {
		self.collected == Some(true) && self.success_files == self.total_files
	}

	pub fn running(self) -> bool {
		self.collected.is_none()
			|| self.success_files + self.failed_files != self.total_files
			|| (self.cleaned.is_none() && self.cooked())
	}

	pub fn success(self) -> bool { self.cleaned == Some(true) && self.cooked() }

	pub fn failed(self) -> bool { self.cleaned == Some(false) || self.collected == Some(false) }

	pub fn cleaned(self) -> Option<bool> { self.cleaned }

	pub fn percent(self) -> Option<f32> {
		Some(if self.success() {
			100.0
		} else if self.failed() {
			0.0
		} else if self.total_bytes != 0 {
			99.99f32.min(self.processed_bytes as f32 / self.total_bytes as f32 * 100.0)
		} else {
			99.99
		})
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
	pub cleaned:         Option<bool>,
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

impl FileProgCut {
	pub fn cooked(self) -> bool {
		self.collected == Some(true) && self.success_files == self.total_files
	}

	pub fn running(self) -> bool {
		self.collected.is_none()
			|| self.success_files + self.failed_files != self.total_files
			|| (self.cleaned.is_none() && self.cooked())
	}

	pub fn success(self) -> bool { self.cleaned == Some(true) && self.cooked() }

	pub fn failed(self) -> bool { self.cleaned == Some(false) || self.collected == Some(false) }

	pub fn cleaned(self) -> Option<bool> { self.cleaned }

	pub fn percent(self) -> Option<f32> {
		Some(if self.success() {
			100.0
		} else if self.failed() {
			0.0
		} else if self.total_bytes != 0 {
			99.99f32.min(self.processed_bytes as f32 / self.total_bytes as f32 * 100.0)
		} else {
			99.99
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
	pub fn cooked(self) -> bool { self.state == Some(true) }

	pub fn running(self) -> bool { self.state.is_none() }

	pub fn success(self) -> bool { self.cooked() }

	pub fn failed(self) -> bool { self.state == Some(false) }

	pub fn cleaned(self) -> Option<bool> { None }

	pub fn percent(self) -> Option<f32> { None }
}

// --- Hardlink
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgHardlink {
	pub total:     u32,
	pub success:   u32,
	pub failed:    u32,
	pub collected: Option<bool>,
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
	pub fn cooked(self) -> bool { self.collected == Some(true) && self.success == self.total }

	pub fn running(self) -> bool {
		self.collected.is_none() || self.success + self.failed != self.total
	}

	pub fn success(self) -> bool { self.cooked() }

	pub fn failed(self) -> bool { self.collected == Some(false) }

	pub fn cleaned(self) -> Option<bool> { None }

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
	pub collected:       Option<bool>,
	pub cleaned:         Option<bool>,
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
	pub fn cooked(self) -> bool {
		self.collected == Some(true) && self.success_files == self.total_files
	}

	pub fn running(self) -> bool {
		self.collected.is_none()
			|| self.success_files + self.failed_files != self.total_files
			|| (self.cleaned.is_none() && self.cooked())
	}

	pub fn success(self) -> bool { self.cleaned == Some(true) && self.cooked() }

	pub fn failed(self) -> bool { self.cleaned == Some(false) || self.collected == Some(false) }

	pub fn cleaned(self) -> Option<bool> { self.cleaned }

	pub fn percent(self) -> Option<f32> {
		Some(if self.success() {
			100.0
		} else if self.failed() {
			0.0
		} else if self.total_bytes != 0 {
			99.99f32.min(self.processed_bytes as f32 / self.total_bytes as f32 * 100.0)
		} else {
			99.99
		})
	}
}

// --- Trash
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileProgTrash {
	pub state:   Option<bool>,
	pub cleaned: Option<bool>,
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
	pub fn cooked(self) -> bool { self.state == Some(true) }

	pub fn running(self) -> bool { self.state.is_none() || (self.cleaned.is_none() && self.cooked()) }

	pub fn success(self) -> bool { self.cleaned == Some(true) && self.cooked() }

	pub fn failed(self) -> bool { self.cleaned == Some(false) || self.state == Some(false) }

	pub fn cleaned(self) -> Option<bool> { self.cleaned }

	pub fn percent(self) -> Option<f32> { None }
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
	pub cleaned:         Option<bool>,
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

impl FileProgDownload {
	pub fn cooked(self) -> bool {
		self.collected == Some(true) && self.success_files == self.total_files
	}

	pub fn running(self) -> bool {
		self.collected.is_none()
			|| self.success_files + self.failed_files != self.total_files
			|| (self.cleaned.is_none() && self.cooked())
	}

	pub fn success(self) -> bool { self.cleaned == Some(true) && self.cooked() }

	pub fn failed(self) -> bool { self.cleaned == Some(false) || self.collected == Some(false) }

	pub fn cleaned(self) -> Option<bool> { self.cleaned }

	pub fn percent(self) -> Option<f32> {
		Some(if self.success() {
			100.0
		} else if self.failed() {
			0.0
		} else if self.total_bytes != 0 {
			99.99f32.min(self.processed_bytes as f32 / self.total_bytes as f32 * 100.0)
		} else {
			99.99
		})
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

impl FileProgUpload {
	pub fn cooked(self) -> bool {
		self.collected == Some(true) && self.success_files == self.total_files
	}

	pub fn running(self) -> bool {
		self.collected.is_none() || self.success_files + self.failed_files != self.total_files
	}

	pub fn success(self) -> bool { self.cooked() }

	pub fn failed(self) -> bool { self.collected == Some(false) }

	pub fn cleaned(self) -> Option<bool> { None }

	pub fn percent(self) -> Option<f32> {
		Some(if self.success() {
			100.0
		} else if self.failed() {
			0.0
		} else if self.total_bytes != 0 {
			99.99f32.min(self.processed_bytes as f32 / self.total_bytes as f32 * 100.0)
		} else {
			99.99
		})
	}
}
