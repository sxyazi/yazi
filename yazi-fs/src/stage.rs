use serde::{Deserialize, Serialize, ser::SerializeMap};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FolderStage {
	#[default]
	Loading,
	Loaded,
	Failed(std::io::ErrorKind),
}

impl FolderStage {
	#[inline]
	pub fn is_loading(self) -> bool { self == Self::Loading }
}

impl Serialize for FolderStage {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut map = serializer.serialize_map(Some(2))?;
		match self {
			Self::Loading => map.serialize_entry("state", "loading")?,
			Self::Loaded => map.serialize_entry("state", "loaded")?,
			Self::Failed(_) => map.serialize_entry("state", "failed")?,
		}
		map.end()
	}
}

impl<'de> Deserialize<'de> for FolderStage {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		#[derive(Deserialize)]
		struct Shadow {
			state: String,
		}

		let shadow = Shadow::deserialize(deserializer)?;
		match shadow.state.as_str() {
			"loading" => Ok(Self::Loading),
			"loaded" => Ok(Self::Loaded),
			"failed" => Ok(Self::Failed(std::io::ErrorKind::Other)),
			_ => Err(serde::de::Error::custom("invalid folder stage")),
		}
	}
}
