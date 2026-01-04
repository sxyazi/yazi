use std::{collections::HashMap, fmt};

use serde::{Deserialize, Deserializer, Serialize, de::Visitor, ser::SerializeStruct};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Attrs {
	pub size:     Option<u64>,
	pub uid:      Option<u32>,
	pub gid:      Option<u32>,
	pub perm:     Option<u32>,
	pub atime:    Option<u32>,
	pub mtime:    Option<u32>,
	pub extended: HashMap<String, String>,
}

impl Attrs {
	pub fn is_empty(&self) -> bool { *self == Self::default() }

	pub fn len(&self) -> usize {
		let mut len = 4;
		if let Some(size) = self.size {
			len += size_of_val(&size);
		}
		if self.uid.is_some() || self.gid.is_some() {
			len += 4 + 4;
		}
		if let Some(perm) = self.perm {
			len += size_of_val(&perm);
		}
		if self.atime.is_some() || self.mtime.is_some() {
			len += 4 + 4;
		}
		if !self.extended.is_empty() {
			len += 4 + self.extended.iter().map(|(k, v)| 4 + k.len() + 4 + v.len()).sum::<usize>();
		}
		len
	}
}

impl Serialize for Attrs {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut flags: u32 = 0;
		let mut len = 1;

		if self.size.is_some() {
			flags |= 0x1;
			len += 1;
		}
		if self.uid.is_some() || self.gid.is_some() {
			flags |= 0x2;
			len += 2;
		}
		if self.perm.is_some() {
			flags |= 0x4;
			len += 1;
		}
		if self.atime.is_some() || self.mtime.is_some() {
			flags |= 0x8;
			len += 2;
		}
		if !self.extended.is_empty() {
			flags |= 0x80000000;
			len += 1 + self.extended.len() * 2;
		}

		let mut seq = serializer.serialize_struct("Attrs", len)?;
		seq.serialize_field("flags", &flags)?;
		if let Some(size) = self.size {
			seq.serialize_field("size", &size)?;
		}
		if self.uid.is_some() || self.gid.is_some() {
			seq.serialize_field("uid", &self.uid.unwrap_or(0))?;
			seq.serialize_field("gid", &self.gid.unwrap_or(0))?;
		}
		if let Some(perm) = self.perm {
			seq.serialize_field("perm", &perm)?;
		}
		if self.atime.is_some() || self.mtime.is_some() {
			seq.serialize_field("atime", &self.atime.unwrap_or(0))?;
			seq.serialize_field("mtime", &self.mtime.unwrap_or(0))?;
		}
		if !self.extended.is_empty() {
			let count = self.extended.len() as u32;
			seq.serialize_field("extended_count", &count)?;
			for (k, v) in self.extended.iter().take(count as usize) {
				seq.serialize_field("extended_key", k)?;
				seq.serialize_field("extended_value", v)?;
			}
		}
		seq.end()
	}
}

impl<'de> Deserialize<'de> for Attrs {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct AttrsVisitor;

		impl<'de> Visitor<'de> for AttrsVisitor {
			type Value = Attrs;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("attributes")
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::SeqAccess<'de>,
			{
				let b = seq.next_element::<u32>()?.unwrap_or(0);
				let mut attrs = Attrs::default();

				if b & 0x1 != 0 {
					attrs.size = seq.next_element()?;
				}
				if b & 0x2 != 0 {
					attrs.uid = seq.next_element()?;
					attrs.gid = seq.next_element()?;
				}
				if b & 0x4 != 0 {
					attrs.perm = seq.next_element()?;
				}
				if b & 0x8 != 0 {
					attrs.atime = seq.next_element()?;
					attrs.mtime = seq.next_element()?;
				}
				if b & 0x80000000 != 0 {
					let count: u32 = seq.next_element()?.unwrap_or(0);
					for _ in 0..count {
						attrs.extended.insert(
							seq.next_element()?.unwrap_or_default(),
							seq.next_element()?.unwrap_or_default(),
						);
					}
				}

				Ok(attrs)
			}
		}

		deserializer.deserialize_any(AttrsVisitor)
	}
}
