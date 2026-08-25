use serde::{Deserialize, Serialize};

use crate::{AsSftpPath, SftpPath};

#[derive(Debug, Deserialize, Serialize)]
pub struct Rename<'a> {
	pub(crate) id: u32,
	from:          SftpPath<'a>,
	to:            SftpPath<'a>,
}

impl<'a> Rename<'a> {
	pub(crate) fn new<F, T>(from: F, to: T) -> Self
	where
		F: AsSftpPath<'a>,
		T: AsSftpPath<'a>,
	{
		Self { id: 0, from: from.as_sftp_path(), to: to.as_sftp_path() }
	}

	pub(crate) fn len(&self) -> usize {
		size_of_val(&self.id) + 4 + self.from.len() + 4 + self.to.len()
	}
}
