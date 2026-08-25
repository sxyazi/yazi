use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize};
use yazi_codegen::DeserializeOver2;
use yazi_fs::{Xdg, create_owned_dir_blocking, path::sanitize_path};
use yazi_shared::timestamp_us;
use yazi_shim::toml::DeserializeOverHook;

use super::PreviewWrap;

#[derive(Debug, Deserialize, DeserializeOver2, Serialize)]
pub struct Preview {
	pub wrap:       PreviewWrap,
	pub tab_size:   u8,
	pub max_width:  u16,
	pub max_height: u16,

	#[serde(deserialize_with = "deserialize_cache_dir")]
	pub cache_dir: PathBuf,

	#[serde(deserialize_with = "deserialize_image_delay")]
	pub image_delay:   u8,
	pub image_filter:  String,
	#[serde(deserialize_with = "deserialize_image_quality")]
	pub image_quality: u8,

	pub ueberzug_scale:  f32,
	pub ueberzug_offset: (f32, f32, f32, f32),
}

impl Preview {
	pub fn tmpfile(&self, prefix: &str) -> PathBuf {
		self.cache_dir.join(format!("{prefix}-{}", timestamp_us()))
	}
}

impl DeserializeOverHook for Preview {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		create_owned_dir_blocking(&self.cache_dir)
			.context(format!("Failed to create cache directory: {}", self.cache_dir.display()))
			.map_err(|err| serde::de::Error::custom(format!("{err:#}")))?;

		Ok(self)
	}
}

fn deserialize_cache_dir<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
	D: Deserializer<'de>,
{
	let path = PathBuf::deserialize(deserializer)?;
	if path.as_os_str().is_empty() {
		Ok(Xdg::temp_dir().to_owned())
	} else {
		sanitize_path(path).ok_or_else(|| {
			serde::de::Error::custom("cache_dir must be either empty or an absolute path.")
		})
	}
}

fn deserialize_image_delay<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
	D: Deserializer<'de>,
{
	let value = u8::deserialize(deserializer)?;
	if value <= 100 {
		Ok(value)
	} else {
		Err(serde::de::Error::custom("image_delay must be between 0 and 100."))
	}
}

fn deserialize_image_quality<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
	D: Deserializer<'de>,
{
	let value = u8::deserialize(deserializer)?;
	if (50..=90).contains(&value) {
		Ok(value)
	} else {
		Err(serde::de::Error::custom("image_quality must be between 50 and 90."))
	}
}
