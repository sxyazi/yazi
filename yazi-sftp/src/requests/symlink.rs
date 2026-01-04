use serde::{Deserialize, Serialize};

use crate::{AsSftpPath, SftpPath};

#[derive(Debug, Deserialize, Serialize)]
pub struct Symlink<'a> {
	pub id:       u32,
	pub link:     SftpPath<'a>,
	pub original: SftpPath<'a>,
}

impl<'a> Symlink<'a> {
	pub fn new<L, O>(link: L, original: O) -> Self
	where
		L: AsSftpPath<'a>,
		O: AsSftpPath<'a>,
	{
		Self { id: 0, link: link.as_sftp_path(), original: original.as_sftp_path() }
	}

	pub fn len(&self) -> usize {
		size_of_val(&self.id) + 4 + self.link.len() + 4 + self.original.len()
	}
}
