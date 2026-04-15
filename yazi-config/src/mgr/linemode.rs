use std::ops::Deref;

use serde::{Deserialize, Deserializer, de};

#[derive(Debug)]
pub struct MgrLinemode(String);

impl Deref for MgrLinemode {
	type Target = String;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'de> Deserialize<'de> for MgrLinemode {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let s = String::deserialize(deserializer)?;
		if s.is_empty() || s.len() > 20 {
			return Err(de::Error::custom("linemode must be between 1 and 20 characters."));
		}

		Ok(Self(s))
	}
}
