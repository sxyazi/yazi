use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::fs::Attrs;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Name<'a> {
	pub(crate) id:    u32,
	pub(crate) items: Vec<NameItem<'a>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct NameItem<'a> {
	pub(crate) name:      Cow<'a, [u8]>,
	pub(crate) long_name: Cow<'a, [u8]>,
	pub(crate) attrs:     Attrs,
}

impl Name<'_> {
	pub(crate) fn len(&self) -> usize {
		size_of_val(&self.id) + 4 + self.items.iter().map(|v| v.len()).sum::<usize>()
	}
}

impl NameItem<'_> {
	fn len(&self) -> usize { 4 + self.name.len() + 4 + self.long_name.len() + self.attrs.len() }
}
