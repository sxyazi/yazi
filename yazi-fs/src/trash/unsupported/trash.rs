use std::{io, path::Path};

use super::super::{TrashEntries, TrashEntry, TrashId};
use crate::{cha::Cha, file::File};

pub struct Trash;

impl Trash {
	pub fn new() -> io::Result<Self> { Ok(Self) }

	pub fn list(&self, _entry: Option<&TrashEntry>) -> io::Result<Vec<TrashEntry>> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn entry(&self, _id: &TrashId) -> io::Result<TrashEntry> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn metadata(&self, _entry: &TrashEntry, _: bool) -> io::Result<Cha> {
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

	pub fn rename(&self, _entry: &TrashEntry, _path: &Path) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn empty(&self) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}
}
