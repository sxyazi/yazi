use serde::{Deserialize, Serialize};

use crate::{AsSftpPath, SftpPath};

#[derive(Debug, Deserialize, Serialize)]
pub struct Rename<'a> {
	pub id:   u32,
	pub from: SftpPath<'a>,
	pub to:   SftpPath<'a>,
}

impl<'a> Rename<'a> {
	pub fn new<F, T>(from: F, to: T) -> Self
	where
		F: AsSftpPath<'a>,
		T: AsSftpPath<'a>,
	{
		Self { id: 0, from: from.as_sftp_path(), to: to.as_sftp_path() }
	}

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.from.len() + 4 + self.to.len() }
}
