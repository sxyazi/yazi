use std::{borrow::Cow, ffi::{OsStr, OsString}, ops::Deref, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ByteStr<'a>(Cow<'a, [u8]>);

impl Deref for ByteStr<'_> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'a> From<&'a str> for ByteStr<'a> {
	fn from(value: &'a str) -> Self { ByteStr(Cow::Borrowed(value.as_bytes())) }
}

impl<'a> From<&'a ByteStr<'a>> for ByteStr<'a> {
	fn from(value: &'a ByteStr) -> Self { ByteStr(Cow::Borrowed(&value.0)) }
}

impl<'a> From<&'a OsStr> for ByteStr<'a> {
	fn from(value: &'a OsStr) -> Self {
		#[cfg(unix)]
		{
			use std::os::unix::ffi::OsStrExt;
			ByteStr(Cow::Borrowed(value.as_bytes()))
		}
		#[cfg(windows)]
		{
			use os_str_bytes::OsStrBytes;
			ByteStr(value.to_raw_bytes())
		}
	}
}

impl<'a> From<&'a Path> for ByteStr<'a> {
	fn from(value: &'a Path) -> Self { ByteStr::from(value.as_os_str()) }
}

impl<'a, T> From<&'a T> for ByteStr<'a>
where
	T: AsRef<Path>,
{
	fn from(value: &'a T) -> Self { Self::from(value.as_ref()) }
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
			use os_str_bytes::OsStrBytes;
			OsStr::assert_from_raw_bytes(self.0.as_ref())
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
			use os_str_bytes::OsStrBytes;
			OsStr::assert_from_raw_bytes(self.0).into_owned()
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

	pub fn join(&self, other: impl Into<ByteStr<'a>>) -> PathBuf {
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
