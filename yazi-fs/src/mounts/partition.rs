use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, Default)]
pub struct Partition {
	pub src:       OsString,
	pub dist:      Option<PathBuf>,
	#[cfg(unix)]
	pub rdev:      Option<libc::dev_t>,
	pub label:     Option<OsString>,
	pub fstype:    Option<OsString>,
	pub capacity:  u64,
	pub external:  Option<bool>,
	pub removable: Option<bool>,
}

impl Partition {
	pub fn heuristic(&self) -> bool {
		let b: &[u8] = self.fstype.as_ref().map_or(b"", |s| s.as_encoded_bytes());
		!matches!(b, b"exfat" | b"fuse.rclone")
	}

	#[rustfmt::skip]
	pub fn systemic(&self) -> bool {
		let _b: &[u8] = self.fstype.as_ref().map_or(b"", |s| s.as_encoded_bytes());
		#[cfg(target_os = "linux")]
		{
			matches!(_b, b"autofs" | b"binfmt_misc" | b"bpf" | b"cgroup2" | b"configfs" | b"debugfs" | b"devpts" | b"devtmpfs" | b"fuse.gvfsd-fuse" | b"fusectl" | b"hugetlbfs" | b"mqueue" | b"proc" | b"pstore" | b"ramfs" | b"securityfs" | b"sysfs" | b"tmpfs" | b"tracefs")
		}
		#[cfg(target_os = "macos")]
		{
			_b.is_empty()
		}
		#[cfg(not(any(target_os = "linux", target_os = "macos")))]
		{
			false
		}
	}
}

impl Partition {
	#[inline]
	#[cfg(any(target_os = "linux", target_os = "macos"))]
	pub(super) fn new(name: &std::ffi::OsStr) -> Self {
		Self { src: std::path::Path::new("/dev/").join(name).into(), ..Default::default() }
	}

	#[inline]
	#[cfg(target_os = "linux")]
	pub(super) fn dev_name(&self, full: bool) -> Option<&std::ffi::OsStr> {
		use std::os::unix::ffi::OsStrExt;

		let s = std::path::Path::new(&self.src).strip_prefix("/dev/").ok()?.as_os_str();
		if full {
			return Some(s);
		}

		let b = s.as_bytes();
		if b.len() < 3 {
			None
		} else if b.starts_with(b"sd") || b.starts_with(b"hd") || b.starts_with(b"vd") {
			Some(std::ffi::OsStr::from_bytes(&b[..3]))
		} else if b.starts_with(b"nvme") || b.starts_with(b"mmcblk") {
			let n = b.iter().position(|&b| b == b'p').unwrap_or(b.len());
			Some(std::ffi::OsStr::from_bytes(&b[..n]))
		} else {
			None
		}
	}
}
