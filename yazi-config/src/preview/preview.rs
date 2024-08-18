use std::{borrow::Cow, path::PathBuf, str::FromStr, time::{SystemTime, UNIX_EPOCH}};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use validator::Validate;
use yazi_shared::fs::expand_path;

use crate::Xdg;

#[derive(Debug, Serialize)]
pub struct Preview {
	pub tab_size:   u8,
	pub max_width:  u32,
	pub max_height: u32,

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
		let time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
		self.cache_dir.join(format!("{prefix}-{}", time.as_nanos() / 1000))
	}

	#[inline]
	pub fn indent(&self) -> Cow<'static, str> {
		match self.tab_size {
			0 => Cow::Borrowed(""),
			1 => Cow::Borrowed(" "),
			2 => Cow::Borrowed("  "),
			3 => Cow::Borrowed("   "),
			4 => Cow::Borrowed("    "),
			5 => Cow::Borrowed("     "),
			6 => Cow::Borrowed("      "),
			7 => Cow::Borrowed("       "),
			8 => Cow::Borrowed("        "),
			n => Cow::Owned(" ".repeat(n as usize)),
		}
	}
}

impl FromStr for Preview {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		#[derive(Deserialize)]
		struct Outer {
			preview: Shadow,
		}
		#[derive(Deserialize, Validate)]
		struct Shadow {
			tab_size:   u8,
			max_width:  u32,
			max_height: u32,

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

		let preview = toml::from_str::<Outer>(s)?.preview;
		preview.validate()?;

		let cache_dir =
			preview.cache_dir.filter(|p| !p.is_empty()).map_or_else(Xdg::cache_dir, expand_path);
		std::fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

		Ok(Preview {
			tab_size: preview.tab_size,
			max_width: preview.max_width,
			max_height: preview.max_height,

			cache_dir,

			image_delay: preview.image_delay,
			image_filter: preview.image_filter,
			image_quality: preview.image_quality,
			sixel_fraction: preview.sixel_fraction,

			ueberzug_scale: preview.ueberzug_scale,
			ueberzug_offset: preview.ueberzug_offset,
		})
	}
}
