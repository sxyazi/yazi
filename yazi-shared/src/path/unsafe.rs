use std::ffi::OsString;

// --- PathUnsafeExt
pub trait PathUnsafeExt<'p> {
	unsafe fn from_encoded_bytes(bytes: &'p [u8]) -> Self;
}

impl<'p> PathUnsafeExt<'p> for &'p std::path::Path {
	unsafe fn from_encoded_bytes(bytes: &'p [u8]) -> Self {
		std::path::Path::new(unsafe { std::ffi::OsStr::from_encoded_bytes_unchecked(bytes) })
	}
}

// --- PathBufUnsafeExt
pub trait PathBufUnsafeExt {
	unsafe fn from_encoded_bytes(bytes: Vec<u8>) -> Self;
}

impl PathBufUnsafeExt for std::path::PathBuf {
	unsafe fn from_encoded_bytes(bytes: Vec<u8>) -> Self {
		Self::from(unsafe { OsString::from_encoded_bytes_unchecked(bytes) })
	}
}
