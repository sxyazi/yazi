use std::io;

use super::{TrashEntry, TrashNode, TrashNodes};
use crate::{cha::Cha, file::File};

pub struct Trash;

impl Trash {
	pub fn new() -> io::Result<Self> { Ok(Self) }

	pub fn list(&self, _node: Option<&TrashNode>) -> io::Result<Vec<TrashEntry>> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn entry(&self, _node: &TrashNode) -> io::Result<TrashEntry> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn metadata(&self, _node: &TrashNode, _: bool) -> io::Result<Cha> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub(super) fn revalidate(
		&self,
		_node: Option<&TrashNode>,
		_current: &File,
	) -> io::Result<Option<File>> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn remove_file(&self, _node: &TrashNode) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn remove_dir(&self, _node: &TrashNode) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn restore(&self, _nodes: TrashNodes) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}

	pub fn empty(&self) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "trash is not supported on this platform"))
	}
}
