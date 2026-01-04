use std::{borrow::Cow, fmt, ops::Deref};

use serde::{Deserialize, Serialize, de::Visitor, ser::SerializeSeq};

use crate::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct Extended<'a> {
	pub id:   u32,
	pub data: ExtendedData<'a>,
}

impl<'a> Extended<'a> {
	pub fn len(&self) -> usize { size_of_val(&self.id) + self.data.len() }
}

// --- Data
#[derive(Debug)]
pub struct ExtendedData<'a>(Cow<'a, [u8]>);

impl Deref for ExtendedData<'_> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Serialize for ExtendedData<'_> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut seq = serializer.serialize_seq(None)?;
		for b in &*self.0 {
			seq.serialize_element(b)?;
		}
		seq.end()
	}
}

impl<'de> Deserialize<'de> for ExtendedData<'_> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct ExtendedDataVisitor;

		impl<'de> Visitor<'de> for ExtendedDataVisitor {
			type Value = Vec<u8>;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("extended data")
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::SeqAccess<'de>,
			{
				let mut bytes = Vec::with_capacity(seq.size_hint().unwrap_or(0));
				while let Some(b) = seq.next_element()? {
					bytes.push(b);
				}
				Ok(bytes)
			}
		}

		deserializer.deserialize_any(ExtendedDataVisitor).map(Cow::Owned).map(ExtendedData)
	}
}

// --- Limits
#[derive(Debug, Deserialize, Serialize)]
pub struct ExtendedLimits {
	pub packet_len:   u64,
	pub read_len:     u64,
	pub write_len:    u64,
	pub open_handles: u64,
}

impl TryFrom<Extended<'_>> for ExtendedLimits {
	type Error = Error;

	fn try_from(value: Extended<'_>) -> Result<Self, Self::Error> {
		crate::Deserializer::once(&value.data)
	}
}
