use std::{ffi::OsStr, io, path::Path};

use super::super::{TrashEntries, TrashEntry, TrashId};
use crate::{file::File, stat::Stat};

pub struct Trash;

impl Trash {
	pub fn new() -> io::Result<Self> { Ok(Self) }

	pub(crate) fn create(
		&self,
		_name: &OsStr,
		_stat: Stat,
		_original: &Path,
	) -> io::Result<TrashEntry> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub(crate) fn rename(&self, _entry: &TrashEntry, _to: &Path) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn list(&self, _entry: Option<&TrashEntry>) -> io::Result<Vec<TrashEntry>> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn entry(&self, _id: &TrashId) -> io::Result<TrashEntry> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub(crate) fn revalidate(
		&self,
		_entry: Option<&TrashEntry>,
		_current: &File,
	) -> io::Result<Option<File>> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn remove_file(&self, _entry: &TrashEntry) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn remove_dir(&self, _entry: &TrashEntry) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn restore(&self, _entries: TrashEntries) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub(crate) fn set_original(_entry: &TrashEntry, _original: &Path) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn empty(&self) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}
}
