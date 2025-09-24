use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{ByteStr, fs::Attrs};

#[derive(Debug, Deserialize, Serialize)]
pub struct SetStat<'a> {
	pub id:    u32,
	pub path:  ByteStr<'a>,
	pub attrs: Attrs,
}

impl<'a> SetStat<'a> {
	pub fn new(path: impl Into<ByteStr<'a>>, attrs: Attrs) -> Self {
		Self { id: 0, path: path.into(), attrs }
	}

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.path.len() + self.attrs.len() }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FSetStat<'a> {
	pub id:     u32,
	pub handle: Cow<'a, str>,
	pub attrs:  Cow<'a, Attrs>,
}

impl<'a> FSetStat<'a> {
	pub fn new(handle: impl Into<Cow<'a, str>>, attrs: &'a Attrs) -> Self {
		Self { id: 0, handle: handle.into(), attrs: Cow::Borrowed(attrs) }
	}

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.handle.len() + self.attrs.len() }
}
