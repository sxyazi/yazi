use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::cha::{Cha, ChaMode};

#[derive(Clone, Copy, Debug, Default)]
pub struct Attrs {
	pub mode:  Option<ChaMode>,
	pub atime: Option<SystemTime>,
	pub btime: Option<SystemTime>,
	pub mtime: Option<SystemTime>,
}

impl From<Cha> for Attrs {
	fn from(value: Cha) -> Self {
		Self { mode: Some(value.mode), atime: value.atime, btime: value.btime, mtime: value.mtime }
	}
}

impl From<Attrs> for std::fs::FileTimes {
	fn from(attrs: Attrs) -> Self {
		let mut t = Self::new();

		if let Some(atime) = attrs.atime {
			t = t.set_accessed(atime);
		}

		#[cfg(target_os = "macos")]
		if let Some(btime) = attrs.btime {
			use std::os::macos::fs::FileTimesExt;
			t = t.set_created(btime);
		}

		#[cfg(windows)]
		if let Some(btime) = attrs.btime {
			use std::os::windows::fs::FileTimesExt;
			t = t.set_created(btime);
		}

		if let Some(mtime) = attrs.mtime {
			t = t.set_modified(mtime);
		}

		t
	}
}

impl Attrs {
	pub fn atime_dur(self) -> Option<Duration> { self.atime?.duration_since(UNIX_EPOCH).ok() }

	pub fn btime_dur(self) -> Option<Duration> { self.btime?.duration_since(UNIX_EPOCH).ok() }

	pub fn mtime_dur(self) -> Option<Duration> { self.mtime?.duration_since(UNIX_EPOCH).ok() }
}
