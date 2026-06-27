use std::fmt;

use serde::{Deserialize, Deserializer, de::{self, Visitor}};

use crate::strand::StrandBuf;

impl<'de> Deserialize<'de> for StrandBuf {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct V;

		impl<'de> Visitor<'de> for V {
			type Value = StrandBuf;

			fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
				f.write_str("a string or bytes")
			}

			fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
				Ok(StrandBuf::Utf8(v.to_owned()))
			}

			fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
				Ok(StrandBuf::Utf8(v))
			}

			fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
				Ok(StrandBuf::Bytes(v.to_owned()))
			}

			fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
				Ok(StrandBuf::Bytes(v))
			}
		}

		deserializer.deserialize_any(V)
	}
}
