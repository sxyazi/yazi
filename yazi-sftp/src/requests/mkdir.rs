use serde::{Deserialize, Serialize};

use crate::{AsSftpPath, SftpPath, fs::Attrs};

#[derive(Debug, Deserialize, Serialize)]
pub struct Mkdir<'a> {
	pub id:    u32,
	pub path:  SftpPath<'a>,
	pub attrs: Attrs,
}

impl<'a> Mkdir<'a> {
	pub fn new<P>(path: P, attrs: Attrs) -> Self
	where
		P: AsSftpPath<'a>,
	{
		Self { id: 0, path: path.as_sftp_path(), attrs }
	}

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.path.len() + self.attrs.len() }
}
