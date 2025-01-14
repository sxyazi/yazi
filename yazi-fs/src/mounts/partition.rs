use std::{ffi::{OsStr, OsString}, path::PathBuf};

#[derive(Debug, Default)]
pub struct Partition {
	pub src:      OsString,
	pub dist:     Option<PathBuf>,
	#[cfg(unix)]
	pub rdev:     libc::dev_t,
	pub label:    OsString,
	pub fstype:   Option<OsString>,
	pub capacity: u64,
}

impl Partition {
	pub fn heuristic(&self) -> bool {
		let b: &[u8] = self.fstype.as_ref().map_or(b"", |s| s.as_encoded_bytes());
		!matches!(b, b"exfat" | b"fuse.rclone")
	}
}

impl Partition {
	#[inline]
	#[cfg(any(target_os = "linux", target_os = "macos"))]
	pub(super) fn new(name: &OsStr) -> Self {
		Self { src: std::path::Path::new("/dev/").join(name).into(), ..Default::default() }
	}

	#[inline]
	#[cfg(target_os = "linux")]
	pub(super) fn dev_name(&self) -> Option<&OsStr> {
		std::path::Path::new(&self.src).strip_prefix("/dev/").ok().map(|p| p.as_os_str())
	}
}
