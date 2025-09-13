use serde::{Deserialize, Serialize};

use crate::{ByteStr, fs::Attrs};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Name<'a> {
	pub id:    u32,
	pub items: Vec<NameItem<'a>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct NameItem<'a> {
	pub name:      ByteStr<'a>,
	pub long_name: ByteStr<'a>,
	pub attrs:     Attrs,
}

impl Name<'_> {
	pub fn len(&self) -> usize {
		size_of_val(&self.id) + 4 + self.items.iter().map(|v| v.len()).sum::<usize>()
	}
}

impl NameItem<'_> {
	pub fn len(&self) -> usize { 4 + self.name.len() + 4 + self.long_name.len() + self.attrs.len() }
}
