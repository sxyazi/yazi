use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize, de};
use yazi_fs::path::sanitize_path;

#[derive(Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ServiceRclone {
	pub remote:      String,
	#[serde(default)]
	pub binary:      PathBuf,
	#[serde(default, deserialize_with = "deserialize_path")]
	pub config_file: PathBuf,
	#[serde(default)]
	pub flags:       Vec<String>,
}

fn deserialize_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
	D: Deserializer<'de>,
{
	let mut path = PathBuf::deserialize(deserializer)?;
	if !path.as_os_str().is_empty() {
		path = sanitize_path(path)
			.ok_or_else(|| de::Error::custom("path must be either empty or an absolute path"))?;
	}

	Ok(path)
}
