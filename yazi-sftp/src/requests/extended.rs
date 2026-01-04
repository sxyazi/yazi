use std::{borrow::Cow, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::{AsSftpPath, SftpPath};

#[derive(Debug, Deserialize, Serialize)]
pub struct Extended<'a, D> {
	pub id:      u32,
	pub request: Cow<'a, str>,
	pub data:    D,
}

impl<D: ExtendedData> Extended<'_, D> {
	pub fn new<'a, R>(request: R, data: D) -> Extended<'a, D>
	where
		R: Into<Cow<'a, str>>,
	{
		Extended { id: 0, request: request.into(), data }
	}

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.request.len() + self.data.len() }
}

// --- Data
pub trait ExtendedData: Debug + Serialize + for<'de> Deserialize<'de> {
	fn len(&self) -> usize;
}

// --- POSIX Rename
#[derive(Debug, Deserialize, Serialize)]
pub struct ExtendedRename<'a> {
	pub from: SftpPath<'a>,
	pub to:   SftpPath<'a>,
}

impl<'a> ExtendedRename<'a> {
	pub fn new<F, T>(from: F, to: T) -> Self
	where
		F: AsSftpPath<'a>,
		T: AsSftpPath<'a>,
	{
		Self { from: from.as_sftp_path(), to: to.as_sftp_path() }
	}
}

impl ExtendedData for ExtendedRename<'_> {
	fn len(&self) -> usize { 4 + self.from.len() + 4 + self.to.len() }
}

// --- Fsync
#[derive(Debug, Deserialize, Serialize)]
pub struct ExtendedFsync<'a> {
	pub handle: Cow<'a, str>,
}

impl<'a> ExtendedFsync<'a> {
	pub fn new(handle: impl Into<Cow<'a, str>>) -> Self { Self { handle: handle.into() } }
}

impl ExtendedData for ExtendedFsync<'_> {
	fn len(&self) -> usize { 4 + self.handle.len() }
}

// --- Hardlink
#[derive(Debug, Deserialize, Serialize)]
pub struct ExtendedHardlink<'a> {
	pub original: SftpPath<'a>,
	pub link:     SftpPath<'a>,
}

impl<'a> ExtendedHardlink<'a> {
	pub fn new<O, L>(original: O, link: L) -> Self
	where
		O: AsSftpPath<'a>,
		L: AsSftpPath<'a>,
	{
		Self { original: original.as_sftp_path(), link: link.as_sftp_path() }
	}
}

impl ExtendedData for ExtendedHardlink<'_> {
	fn len(&self) -> usize { 4 + self.original.len() + 4 + self.link.len() }
}

// --- Limits
#[derive(Debug, Deserialize, Serialize)]
pub struct ExtendedLimits;

impl ExtendedData for ExtendedLimits {
	fn len(&self) -> usize { 0 }
}
