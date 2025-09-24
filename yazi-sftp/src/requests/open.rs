use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{ByteStr, fs::{Attrs, Flags}};

#[derive(Debug, Deserialize, Serialize)]
pub struct Open<'a> {
	pub id:    u32,
	pub path:  ByteStr<'a>,
	pub flags: Flags,
	pub attrs: Cow<'a, Attrs>,
}

impl<'a> Open<'a> {
	pub fn new<P>(path: P, flags: Flags, attrs: &'a Attrs) -> Self
	where
		P: Into<ByteStr<'a>>,
	{
		Self { id: 0, path: path.into(), flags, attrs: Cow::Borrowed(attrs) }
	}

	pub fn len(&self) -> usize {
		size_of_val(&self.id) + 4 + self.path.len() + size_of_val(&self.flags) + self.attrs.len()
	}
}
