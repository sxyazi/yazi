use std::ffi::{OsStr, OsString};

use anyhow::Result;

// --- AsWtf8
pub trait AsWtf8 {
	fn as_wtf8(&self) -> &[u8];
}

impl AsWtf8 for OsStr {
	fn as_wtf8(&self) -> &[u8] {
		#[cfg(unix)]
		{
			use std::os::unix::ffi::OsStrExt;
			self.as_bytes()
		}
		#[cfg(windows)]
		{
			self.as_encoded_bytes()
		}
	}
}

// --- FromWtf8
pub trait FromWtf8 {
	fn from_wtf8(wtf8: &[u8]) -> Result<&Self>;
}

impl FromWtf8 for OsStr {
	fn from_wtf8(wtf8: &[u8]) -> Result<&Self> {
		#[cfg(unix)]
		{
			use std::os::unix::ffi::OsStrExt;
			Ok(OsStr::from_bytes(wtf8))
		}
		#[cfg(windows)]
		{
			// FIXME: validate WTF-8
			Ok(unsafe { OsStr::from_encoded_bytes_unchecked(wtf8) })
		}
	}
}

impl FromWtf8 for std::path::Path {
	fn from_wtf8(wtf8: &[u8]) -> Result<&Self> { Ok(OsStr::from_wtf8(wtf8)?.as_ref()) }
}

// --- FromWtf8Vec
pub trait FromWtf8Vec {
	fn from_wtf8_vec(wtf8: Vec<u8>) -> Result<Self>
	where
		Self: Sized;
}

impl FromWtf8Vec for OsString {
	fn from_wtf8_vec(wtf8: Vec<u8>) -> Result<Self> {
		#[cfg(unix)]
		{
			use std::os::unix::ffi::OsStringExt;
			Ok(OsString::from_vec(wtf8))
		}
		#[cfg(windows)]
		{
			// FIXME: validate WTF-8
			Ok(unsafe { OsString::from_encoded_bytes_unchecked(wtf8) })
		}
	}
}

impl FromWtf8Vec for std::path::PathBuf {
	fn from_wtf8_vec(wtf8: Vec<u8>) -> Result<Self> { Ok(OsString::from_wtf8_vec(wtf8)?.into()) }
}
