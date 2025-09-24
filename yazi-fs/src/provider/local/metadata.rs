use std::fs::{FileTimes, Permissions};

use crate::cha::Cha;

impl From<Cha> for FileTimes {
	fn from(cha: Cha) -> Self {
		let mut t = std::fs::FileTimes::new();

		if let Some(atime) = cha.atime {
			t = t.set_accessed(atime);
		}

		#[cfg(target_os = "macos")]
		if let Some(btime) = cha.btime {
			use std::os::macos::fs::FileTimesExt;
			t = t.set_created(btime);
		}

		#[cfg(windows)]
		if let Some(btime) = cha.btime {
			use std::os::windows::fs::FileTimesExt;
			t = t.set_created(btime);
		}

		if let Some(mtime) = cha.mtime {
			t = t.set_modified(mtime);
		}

		t
	}
}

#[cfg(unix)]
impl From<Cha> for Permissions {
	fn from(cha: Cha) -> Self {
		use std::os::unix::fs::PermissionsExt;

		Permissions::from_mode(cha.mode.bits() as _)
	}
}
