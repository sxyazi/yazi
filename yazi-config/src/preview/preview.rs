use std::{borrow::Cow, path::PathBuf, str::FromStr};

use anyhow::Context;
use serde::{Deserialize, Deserializer, Serialize};
use validator::Validate;
use yazi_fs::{Xdg, expand_path};
use yazi_shared::timestamp_us;

use super::{Alignment, PreviewWrap};

#[rustfmt::skip]
const TABS: &[&str] = &["", " ", "  ", "   ", "    ", "     ", "      ", "       ", "        ", "         ", "          ", "           ", "            ", "             ", "              ", "               ", "                "];

#[derive(Debug, Serialize)]
pub struct Preview {
	pub wrap:       PreviewWrap,
	pub tab_size:   u8,
	pub max_width:  u32,
	pub max_height: u32,
	pub alignment:  Alignment,

	pub cache_dir: PathBuf,

	pub image_delay:    u8,
	pub image_filter:   String,
	pub image_quality:  u8,
	pub sixel_fraction: u8,

	pub ueberzug_scale:  f32,
	pub ueberzug_offset: (f32, f32, f32, f32),
}

impl Preview {
	#[inline]
	pub fn tmpfile(&self, prefix: &str) -> PathBuf {
		self.cache_dir.join(format!("{prefix}-{}", timestamp_us()))
	}

	#[inline]
	pub fn indent(&self) -> Cow<'static, str> { Self::indent_with(self.tab_size as usize) }

	#[inline]
	pub fn indent_with(n: usize) -> Cow<'static, str> {
		if let Some(s) = TABS.get(n) { Cow::Borrowed(s) } else { Cow::Owned(" ".repeat(n)) }
	}
}

impl FromStr for Preview {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let preview: Self =
			toml::from_str(s).context("Failed to parse the [preview] section in your yazi.toml")?;

		std::fs::create_dir_all(&preview.cache_dir).context("Failed to create cache directory")?;

		Ok(preview)
	}
}

impl<'de> Deserialize<'de> for Preview {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Outer {
			preview: Shadow,
		}
		#[derive(Deserialize, Validate)]
		struct Shadow {
			wrap:       PreviewWrap,
			tab_size:   u8,
			max_width:  u32,
			max_height: u32,
			#[serde(default)]
			alignment:  Alignment,

			cache_dir: Option<String>,

			#[validate(range(min = 0, max = 100))]
			image_delay:    u8,
			image_filter:   String,
			#[validate(range(min = 50, max = 90))]
			image_quality:  u8,
			#[validate(range(min = 10, max = 20))]
			sixel_fraction: u8,

			ueberzug_scale:  f32,
			ueberzug_offset: (f32, f32, f32, f32),
		}

		let preview = Outer::deserialize(deserializer)?.preview;
		preview.validate().map_err(serde::de::Error::custom)?;

		Ok(Preview {
			wrap:       preview.wrap,
			tab_size:   preview.tab_size,
			max_width:  preview.max_width,
			max_height: preview.max_height,
			alignment:  preview.alignment,

			cache_dir: preview
				.cache_dir
				.filter(|p| !p.is_empty())
				.map_or_else(Xdg::cache_dir, expand_path),

			image_delay:    preview.image_delay,
			image_filter:   preview.image_filter,
			image_quality:  preview.image_quality,
			sixel_fraction: preview.sixel_fraction,

			ueberzug_scale:  preview.ueberzug_scale,
			ueberzug_offset: preview.ueberzug_offset,
		})
	}
}
