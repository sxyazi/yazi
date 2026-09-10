use std::{fs::{self, FileTimes}, io, path::Path};

pub fn set_times(path: &Path, times: FileTimes) -> io::Result<()> {
	#[cfg(not(windows))]
	let file = fs::File::open(path);
	#[cfg(windows)]
	let file = {
		use std::os::windows::fs::OpenOptionsExt;
		fs::File::options()
			.access_mode(windows_sys::Win32::Storage::FileSystem::FILE_WRITE_ATTRIBUTES)
			.open(path)
	};

	file?.set_times(times)
}
