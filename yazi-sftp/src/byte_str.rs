use std::{borrow::Cow, ffi::{OsStr, OsString}, ops::Deref, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ByteStr<'a>(Cow<'a, [u8]>);

impl Deref for ByteStr<'_> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'a> From<&'a str> for ByteStr<'a> {
	fn from(value: &'a str) -> Self { ByteStr(Cow::Borrowed(value.as_bytes())) }
}

impl<'a> From<&'a Self> for ByteStr<'a> {
	fn from(value: &'a ByteStr) -> Self { ByteStr(Cow::Borrowed(&value.0)) }
}

impl PartialEq<&str> for ByteStr<'_> {
	fn eq(&self, other: &&str) -> bool { self.0 == other.as_bytes() }
}

impl<'a> ByteStr<'a> {
	pub fn to_os_str(&self) -> Cow<'_, OsStr> {
		#[cfg(unix)]
		{
			use std::os::unix::ffi::OsStrExt;
			OsStr::from_bytes(&self.0).into()
		}
		#[cfg(windows)]
		{
			super::wtf::bytes_to_wide(self.0.as_ref())
		}
	}

	pub fn into_os_string(self) -> OsString {
		#[cfg(unix)]
		{
			use std::os::unix::ffi::OsStringExt;
			OsString::from_vec(self.0.into_owned())
		}
		#[cfg(windows)]
		{
			match super::wtf::bytes_to_wide(self.0.as_ref()) {
				Cow::Borrowed(_) => unsafe { String::from_utf8_unchecked(self.0.into_owned()) }.into(),
				Cow::Owned(s) => s,
			}
		}
	}

	pub fn to_path(&self) -> Cow<'_, Path> {
		match self.to_os_str() {
			Cow::Borrowed(s) => Path::new(s).into(),
			Cow::Owned(s) => PathBuf::from(s).into(),
		}
	}

	pub fn into_path(self) -> PathBuf {
		match self.0 {
			Cow::Borrowed(_) => self.to_path().into_owned(),
			Cow::Owned(_) => self.into_os_string().into(),
		}
	}

	pub fn join(&self, other: impl Into<Self>) -> PathBuf {
		let other = other.into();
		match self.to_path() {
			Cow::Borrowed(p) => p.join(other.to_path()),
			Cow::Owned(mut p) => {
				p.push(other.to_path());
				p
			}
		}
	}

	pub fn into_owned(self) -> ByteStr<'static> { ByteStr(Cow::Owned(self.0.into_owned())) }

	pub(super) unsafe fn from_str_bytes_unchecked(bytes: &'a [u8]) -> Self {
		Self(Cow::Borrowed(bytes))
	}
}

// --- Traits
pub trait ToByteStr<'a> {
	fn to_byte_str(self) -> Result<ByteStr<'a>, Error>;
}

impl<'a, T> ToByteStr<'a> for &'a T
where
	T: AsRef<Path> + ?Sized,
{
	fn to_byte_str(self) -> Result<ByteStr<'a>, Error> {
		#[cfg(unix)]
		{
			use std::os::unix::ffi::OsStrExt;
			Ok(ByteStr(Cow::Borrowed(self.as_ref().as_os_str().as_bytes())))
		}
		#[cfg(windows)]
		{
			super::wtf::wide_to_bytes(self.as_ref().as_os_str())
				.ok_or(Error::custom("failed to convert wide path to bytes"))
				.map(ByteStr)
		}
	}
}

impl<'a> ToByteStr<'a> for &'a ByteStr<'a> {
	fn to_byte_str(self) -> Result<ByteStr<'a>, Error> { Ok(ByteStr(Cow::Borrowed(&self.0))) }
}
