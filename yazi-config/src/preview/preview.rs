use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize};
use yazi_codegen::DeserializeOver2;
use yazi_fs::Xdg;
use yazi_shared::{SStr, timestamp_us};
use yazi_shim::toml::DeserializeOverHook;

use super::PreviewWrap;
use crate::normalize_path;

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

	pub fn indent(&self) -> SStr {
		#[rustfmt::skip]
		const TABS: &[&str] = &["", " ", "  ", "   ", "    ", "     ", "      ", "       ", "        ", "         ", "          ", "           ", "            ", "             ", "              ", "               ", "                "];

		if let Some(&s) = TABS.get(self.tab_size as usize) {
			s.into()
		} else {
			" ".repeat(self.tab_size as usize).into()
		}
	}
}

impl DeserializeOverHook for Preview {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		create_cache_dir(&self.cache_dir).map_err(serde::de::Error::custom)?;

		Ok(self)
	}
}

fn create_cache_dir(path: &Path) -> Result<()> {
	create_cache_dir_all(path)?;
	restrict_cache_dir_permissions(path)?;
	Ok(())
}

#[cfg(unix)]
fn create_cache_dir_all(path: &Path) -> Result<()> {
	use std::os::unix::fs::DirBuilderExt;

	std::fs::DirBuilder::new()
		.recursive(true)
		.mode(0o700)
		.create(path)
		.context(format!("Failed to create cache directory: {}", path.display()))
}

#[cfg(not(unix))]
fn create_cache_dir_all(path: &Path) -> Result<()> {
	std::fs::create_dir_all(path)
		.context(format!("Failed to create cache directory: {}", path.display()))
}

#[cfg(unix)]
fn restrict_cache_dir_permissions(path: &Path) -> Result<()> {
	use std::os::unix::fs::PermissionsExt;

	std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
		.context(format!("Failed to set cache directory permissions: {}", path.display()))
}

#[cfg(not(unix))]
fn restrict_cache_dir_permissions(_: &Path) -> Result<()> {
	Ok(())
}

fn deserialize_cache_dir<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
	D: Deserializer<'de>,
{
	let path = PathBuf::deserialize(deserializer)?;
	if path.as_os_str().is_empty() {
		Ok(Xdg::temp_dir().to_owned())
	} else {
		normalize_path(path).ok_or_else(|| {
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

#[cfg(test)]
mod tests {
	use super::*;

	#[cfg(unix)]
	#[test]
	fn create_cache_dir_restricts_permissions_on_unix() {
		use std::os::unix::fs::PermissionsExt;

		let cache_dir = std::env::temp_dir().join(format!("yazi-cache-test-{}", timestamp_us()));
		std::fs::create_dir_all(&cache_dir).unwrap();
		std::fs::set_permissions(&cache_dir, std::fs::Permissions::from_mode(0o755)).unwrap();

		create_cache_dir(&cache_dir).unwrap();
		let mode = std::fs::metadata(&cache_dir).unwrap().permissions().mode() & 0o777;
		let _ = std::fs::remove_dir_all(&cache_dir);

		assert_eq!(mode, 0o700);
	}
}
