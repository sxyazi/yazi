use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{AsSftpPath, SftpPath, fs::{Attrs, Flags}};

#[derive(Debug, Deserialize, Serialize)]
pub struct Open<'a> {
	pub(crate) id: u32,
	path:          SftpPath<'a>,
	flags:         Flags,
	attrs:         Cow<'a, Attrs>,
}

impl<'a> Open<'a> {
	pub(crate) fn new<P>(path: P, flags: Flags, attrs: &'a Attrs) -> Self
	where
		P: AsSftpPath<'a>,
	{
		Self { id: 0, path: path.as_sftp_path(), flags, attrs: Cow::Borrowed(attrs) }
	}

	pub(crate) fn len(&self) -> usize {
		size_of_val(&self.id) + 4 + self.path.len() + size_of_val(&self.flags) + self.attrs.len()
	}
}
