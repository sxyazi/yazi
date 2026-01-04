use serde::{Deserialize, Serialize};

use crate::{AsSftpPath, SftpPath};

#[derive(Debug, Deserialize, Serialize)]
pub struct Lstat<'a> {
	pub id:   u32,
	pub path: SftpPath<'a>,
}

impl Lstat<'_> {
	pub fn new<'a, P>(path: P) -> Lstat<'a>
	where
		P: AsSftpPath<'a>,
	{
		Lstat { id: 0, path: path.as_sftp_path() }
	}

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.path.len() }
}
