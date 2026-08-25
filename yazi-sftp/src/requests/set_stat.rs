use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{AsSftpPath, SftpPath, fs::Attrs};

#[derive(Debug, Deserialize, Serialize)]
pub struct SetStat<'a> {
	pub(crate) id: u32,
	path:          SftpPath<'a>,
	attrs:         Attrs,
}

impl<'a> SetStat<'a> {
	pub(crate) fn new<P>(path: P, attrs: Attrs) -> Self
	where
		P: AsSftpPath<'a>,
	{
		Self { id: 0, path: path.as_sftp_path(), attrs }
	}

	pub(crate) fn len(&self) -> usize {
		size_of_val(&self.id) + 4 + self.path.len() + self.attrs.len()
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FSetStat<'a> {
	pub(crate) id: u32,
	handle:        Cow<'a, str>,
	attrs:         Cow<'a, Attrs>,
}

impl<'a> FSetStat<'a> {
	pub(crate) fn new(handle: impl Into<Cow<'a, str>>, attrs: &'a Attrs) -> Self {
		Self { id: 0, handle: handle.into(), attrs: Cow::Borrowed(attrs) }
	}

	pub(crate) fn len(&self) -> usize {
		size_of_val(&self.id) + 4 + self.handle.len() + self.attrs.len()
	}
}
