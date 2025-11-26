use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{AsSftpPath, SftpPath, fs::{Attrs, Flags}};

#[derive(Debug, Deserialize, Serialize)]
pub struct Open<'a> {
	pub id:    u32,
	pub path:  SftpPath<'a>,
	pub flags: Flags,
	pub attrs: Cow<'a, Attrs>,
}

impl<'a> Open<'a> {
	pub fn new<P>(path: P, flags: Flags, attrs: &'a Attrs) -> Self
	where
		P: AsSftpPath<'a>,
	{
		Self { id: 0, path: path.as_sftp_path(), flags, attrs: Cow::Borrowed(attrs) }
	}

	pub fn len(&self) -> usize {
		size_of_val(&self.id) + 4 + self.path.len() + size_of_val(&self.flags) + self.attrs.len()
	}
}
