use std::{path::PathBuf, time::{self, SystemTime}};

use serde::{Deserialize, Serialize};
use validator::Validate;
use yazi_shared::fs::expand_path;

use crate::{validation::check_validation, Xdg, MERGED_YAZI};

#[derive(Debug, Serialize)]
pub struct Preview {
	pub tab_size:   u8,
	pub word_wrap: bool,
	pub max_width:  u32,
	pub max_height: u32,

	pub cache_dir: PathBuf,

	pub image_filter:   String,
	pub image_quality:  u8,
	pub sixel_fraction: u8,

	pub ueberzug_scale:  f32,
	pub ueberzug_offset: (f32, f32, f32, f32),
}

impl Default for Preview {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			preview: Shadow,
		}
		#[derive(Deserialize, Validate)]
		struct Shadow {
			tab_size:   u8,
			word_wrap:  bool,
			max_width:  u32,
			max_height: u32,

			cache_dir: Option<String>,

			image_filter:   String,
			#[validate(range(min = 50, max = 90))]
			image_quality:  u8,
			#[validate(range(min = 10, max = 20))]
			sixel_fraction: u8,

			ueberzug_scale:  f32,
			ueberzug_offset: (f32, f32, f32, f32),
		}

		let preview = toml::from_str::<Outer>(&MERGED_YAZI).unwrap().preview;
		check_validation(preview.validate());

		let cache_dir =
			preview.cache_dir.filter(|p| !p.is_empty()).map_or_else(Xdg::cache_dir, expand_path);
		std::fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");

		Preview {
			tab_size: preview.tab_size,
			word_wrap: preview.word_wrap,
			max_width: preview.max_width,
			max_height: preview.max_height,

			cache_dir,

			image_filter: preview.image_filter,
			image_quality: preview.image_quality,
			sixel_fraction: preview.sixel_fraction,

			ueberzug_scale: preview.ueberzug_scale,
			ueberzug_offset: preview.ueberzug_offset,
		}
	}
}

impl Preview {
	#[inline]
	pub fn tmpfile(&self, prefix: &str) -> PathBuf {
		let nanos = SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_nanos();
		self.cache_dir.join(format!("{prefix}-{}", nanos / 1000))
	}
}
