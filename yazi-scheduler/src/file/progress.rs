use yazi_parser::app::TaskSummary;

// --- Paste
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FileProgPaste {
	pub(crate) total_files:     u32,
	pub(crate) success_files:   u32,
	pub(crate) failed_files:    u32,
	pub(crate) total_bytes:     u64,
	pub(crate) processed_bytes: u64,
	pub(crate) collected:       bool,
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
	pub(crate) fn running(self) -> bool {
		!self.collected || self.success_files + self.failed_files != self.total_files
	}

	pub(crate) fn success(self) -> bool { self.collected && self.success_files == self.total_files }

	pub(crate) fn percent(self) -> Option<f32> {
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
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FileProgLink {
	pub(crate) state: Option<bool>,
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
	pub(crate) fn running(self) -> bool { self.state.is_none() }

	pub(crate) fn success(self) -> bool { self.state == Some(true) }

	pub(crate) fn percent(self) -> Option<f32> { None }
}

// --- Hardlink
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FileProgHardlink {
	pub(crate) total:     u32,
	pub(crate) success:   u32,
	pub(crate) failed:    u32,
	pub(crate) collected: bool,
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
	pub(crate) fn running(self) -> bool {
		!self.collected || self.success + self.failed != self.total
	}

	pub(crate) fn success(self) -> bool { self.collected && self.success == self.total }

	pub(crate) fn percent(self) -> Option<f32> { None }
}

// --- Delete
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FileProgDelete {
	pub(crate) total_files:     u32,
	pub(crate) success_files:   u32,
	pub(crate) failed_files:    u32,
	pub(crate) total_bytes:     u64,
	pub(crate) processed_bytes: u64,
	pub(crate) collected:       bool,
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
	pub(crate) fn running(self) -> bool {
		!self.collected || self.success_files + self.failed_files != self.total_files
	}

	pub(crate) fn success(self) -> bool { self.collected && self.success_files == self.total_files }

	pub(crate) fn percent(self) -> Option<f32> {
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
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FileProgTrash {
	pub(crate) state: Option<bool>,
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
	pub(crate) fn running(self) -> bool { self.state.is_none() }

	pub(crate) fn success(self) -> bool { self.state == Some(true) }

	pub(crate) fn percent(self) -> Option<f32> { None }
}
