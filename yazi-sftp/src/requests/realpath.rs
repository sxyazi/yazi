use serde::{Deserialize, Serialize};

use crate::{AsSftpPath, SftpPath};

#[derive(Debug, Deserialize, Serialize)]
pub struct Realpath<'a> {
	pub id:   u32,
	pub path: SftpPath<'a>,
}

impl<'a> Realpath<'a> {
	pub fn new<P>(path: P) -> Self
	where
		P: AsSftpPath<'a>,
	{
		Self { id: 0, path: path.as_sftp_path() }
	}

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.path.len() }
}
