use serde::{Deserialize, Serialize};

use crate::{AsSftpPath, SftpPath};

#[derive(Debug, Deserialize, Serialize)]
pub struct Symlink<'a> {
	pub(crate) id: u32,
	link:          SftpPath<'a>,
	original:      SftpPath<'a>,
}

impl<'a> Symlink<'a> {
	pub(crate) fn new<L, O>(link: L, original: O) -> Self
	where
		L: AsSftpPath<'a>,
		O: AsSftpPath<'a>,
	{
		Self { id: 0, link: link.as_sftp_path(), original: original.as_sftp_path() }
	}

	pub(crate) fn len(&self) -> usize {
		size_of_val(&self.id) + 4 + self.link.len() + 4 + self.original.len()
	}
}
