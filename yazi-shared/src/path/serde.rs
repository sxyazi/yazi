use std::fmt::{self, Formatter};

use percent_encoding::{AsciiSet, CONTROLS, percent_decode, percent_encode};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::{self, Visitor}};

use super::{DynPath, PathBufDyn, PathCow, PathDyn, PathKind};

impl Serialize for PathDyn<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		const SET: &AsciiSet = &CONTROLS.add(b'%');

		match (self.kind(), self.to_str()) {
			(PathKind::Os, Ok(s)) => serializer.collect_str(&format_args!("o:{s}")),
			(PathKind::Os, Err(_)) => {
				serializer.collect_str(&format_args!("O:{}", percent_encode(self.encoded_bytes(), SET)))
			}
			(PathKind::Unix, Ok(s)) => serializer.collect_str(&format_args!("u:{s}")),
			(PathKind::Unix, Err(_)) => {
				serializer.collect_str(&format_args!("U:{}", percent_encode(self.encoded_bytes(), SET)))
			}
		}
	}
}

impl Serialize for PathBufDyn {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		self.dyn_path().serialize(serializer)
	}
}

impl<'de> Deserialize<'de> for PathBufDyn {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct V;

		impl<'de> Visitor<'de> for V {
			type Value = PathBufDyn;

			fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
				formatter.write_str("a UTF-8 path string")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				let path = if let Some(path) = value.strip_prefix("o:") {
					PathCow::with(PathKind::Os, path.as_bytes())
				} else if let Some(path) = value.strip_prefix("O:") {
					PathCow::with(PathKind::Os, percent_decode(path.as_bytes()))
				} else if let Some(path) = value.strip_prefix("u:") {
					PathCow::with(PathKind::Unix, path.as_bytes())
				} else if let Some(path) = value.strip_prefix("U:") {
					PathCow::with(PathKind::Unix, percent_decode(path.as_bytes()))
				} else {
					return Err(E::custom("invalid PathBufDyn encoding"));
				};

				path.map(PathCow::into_owned).map_err(E::custom)
			}
		}

		deserializer.deserialize_str(V)
	}
}
