use std::{borrow::Cow, ops::Deref};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum SftpPath<'a> {
	Borrowed(&'a typed_path::UnixPath),
	Owned(typed_path::UnixPathBuf),
}

impl Deref for SftpPath<'_> {
	type Target = typed_path::UnixPath;

	fn deref(&self) -> &Self::Target {
		match self {
			SftpPath::Borrowed(p) => p,
			SftpPath::Owned(p) => p.as_path(),
		}
	}
}

impl Default for SftpPath<'_> {
	fn default() -> Self { SftpPath::Borrowed(typed_path::UnixPath::new("")) }
}

impl<'a> From<&'a Self> for SftpPath<'a> {
	fn from(value: &'a SftpPath) -> Self { SftpPath::Borrowed(value) }
}

impl<'a> From<Cow<'a, [u8]>> for SftpPath<'a> {
	fn from(value: Cow<'a, [u8]>) -> Self {
		match value {
			Cow::Borrowed(b) => Self::Borrowed(typed_path::UnixPath::new(b)),
			Cow::Owned(b) => SftpPath::Owned(typed_path::UnixPathBuf::from(b)),
		}
	}
}

impl Serialize for SftpPath<'_> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_bytes(self.as_bytes())
	}
}

impl<'de> Deserialize<'de> for SftpPath<'_> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let cow = <Cow<'de, [u8]>>::deserialize(deserializer)?;
		Ok(Self::Owned(cow.into_owned().into()))
	}
}

impl<'a> SftpPath<'a> {
	pub fn len(&self) -> usize { self.as_bytes().len() }

	pub fn into_owned(self) -> typed_path::UnixPathBuf {
		match self {
			SftpPath::Borrowed(p) => p.to_owned(),
			SftpPath::Owned(p) => p,
		}
	}
}

// --- Traits
pub trait AsSftpPath<'a> {
	fn as_sftp_path(self) -> SftpPath<'a>;
}

impl<'a, T> AsSftpPath<'a> for &'a T
where
	T: ?Sized + AsRef<typed_path::UnixPath>,
{
	fn as_sftp_path(self) -> SftpPath<'a> { SftpPath::Borrowed(self.as_ref()) }
}

impl<'a> AsSftpPath<'a> for &'a SftpPath<'a> {
	fn as_sftp_path(self) -> SftpPath<'a> { SftpPath::Borrowed(self) }
}
