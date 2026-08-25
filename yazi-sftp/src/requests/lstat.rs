use serde::{Deserialize, Serialize};

use crate::{AsSftpPath, SftpPath};

#[derive(Debug, Deserialize, Serialize)]
pub struct Lstat<'a> {
	pub(crate) id: u32,
	path:          SftpPath<'a>,
}

impl Lstat<'_> {
	pub(crate) fn new<'a, P>(path: P) -> Lstat<'a>
	where
		P: AsSftpPath<'a>,
	{
		Lstat { id: 0, path: path.as_sftp_path() }
	}

	pub(crate) fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.path.len() }
}
