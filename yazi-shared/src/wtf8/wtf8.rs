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
			Ok(Self::from_bytes(wtf8))
		}
		#[cfg(windows)]
		{
			if super::valid_wtf8(wtf8) {
				Ok(unsafe { Self::from_encoded_bytes_unchecked(wtf8) })
			} else {
				Err(anyhow::anyhow!("Invalid WTF-8 sequence"))
			}
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
			Ok(Self::from_vec(wtf8))
		}
		#[cfg(windows)]
		{
			if super::valid_wtf8(&wtf8) {
				Ok(unsafe { Self::from_encoded_bytes_unchecked(wtf8) })
			} else {
				Err(anyhow::anyhow!("Invalid WTF-8 sequence"))
			}
		}
	}
}

impl FromWtf8Vec for std::path::PathBuf {
	fn from_wtf8_vec(wtf8: Vec<u8>) -> Result<Self> { Ok(OsString::from_wtf8_vec(wtf8)?.into()) }
}
