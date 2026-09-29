use std::{ffi::OsString, os::windows::ffi::{OsStrExt, OsStringExt}, path::PathBuf};

use windows_sys::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDriveStringsW, GetVolumeInformationW};

use super::Partition;

// GetDriveTypeW return values (winbase.h)
const DRIVE_REMOVABLE: u32 = 2;
const DRIVE_REMOTE: u32 = 4;

/// Enumerate mounted drives (`C:\`, `D:\`, ...) with their volume labels,
/// filesystems and capacities.
///
/// Used to populate the drive picker shown when `leave` is pressed at a
/// drive root - on Windows the filesystem has no single root, so drives act
/// as the topmost level of navigation (issue #3240).
pub fn drives() -> Vec<Partition> {
	let mut buf = [0u16; 512];
	let n = unsafe { GetLogicalDriveStringsW(buf.len() as _, buf.as_mut_ptr()) };
	if n == 0 || n as usize >= buf.len() {
		return Vec::new();
	}

	buf[..n as usize]
		.split(|&c| c == 0)
		.filter(|s| !s.is_empty())
		.map(|s| drive(OsString::from_wide(s)))
		.collect()
}

fn drive(root: OsString) -> Partition {
	let root_w: Vec<u16> = root.encode_wide().chain(Some(0)).collect();
	let ty = unsafe { GetDriveTypeW(root_w.as_ptr()) };

	let mut label = [0u16; 261];
	let mut fstype = [0u16; 261];
	let has_info = unsafe {
		GetVolumeInformationW(
			root_w.as_ptr(),
			label.as_mut_ptr(),
			label.len() as _,
			std::ptr::null_mut(),
			std::ptr::null_mut(),
			std::ptr::null_mut(),
			fstype.as_mut_ptr(),
			fstype.len() as _,
		) != 0
	};

	let mut capacity = 0u64;
	unsafe {
		GetDiskFreeSpaceExW(root_w.as_ptr(), std::ptr::null_mut(), &mut capacity, std::ptr::null_mut())
	};

	Partition {
		src: root.clone(),
		dist: Some(PathBuf::from(&root)),
		label: if has_info { os(&label) } else { None },
		fstype: if has_info { os(&fstype) } else { None },
		capacity,
		external: Some(ty == DRIVE_REMOTE),
		removable: Some(ty == DRIVE_REMOVABLE),
		..Default::default()
	}
}

fn os(buf: &[u16]) -> Option<OsString> {
	let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
	match end {
		0 => None,
		n => Some(OsString::from_wide(&buf[..n])),
	}
}
