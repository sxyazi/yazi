use std::{borrow::Cow, path::PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use yazi_codegen::DeserializeOver2;
use yazi_fs::{Xdg, expand_path};
use yazi_shared::timestamp_us;

use super::PreviewWrap;

#[rustfmt::skip]
const TABS: &[&str] = &["", " ", "  ", "   ", "    ", "     ", "      ", "       ", "        ", "         ", "          ", "           ", "            ", "             ", "              ", "               ", "                "];

#[derive(Debug, Deserialize, DeserializeOver2, Serialize)]
pub struct Preview {
	pub wrap:       PreviewWrap,
	pub tab_size:   u8,
	pub max_width:  u32,
	pub max_height: u32,

	pub cache_dir: PathBuf,

	pub image_delay:   u8,
	pub image_filter:  String,
	pub image_quality: u8,

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

impl Preview {
	pub(crate) fn reshape(mut self) -> Result<Self> {
		if self.image_delay > 100 {
			bail!("[preview].image_delay must be between 0 and 100.");
		} else if self.image_quality < 50 || self.image_quality > 90 {
			bail!("[preview].image_quality must be between 50 and 90.");
		}

		self.cache_dir = if self.cache_dir.as_os_str().is_empty() {
			Xdg::cache_dir()
		} else {
			expand_path(&self.cache_dir)
		};

		std::fs::create_dir_all(&self.cache_dir).context("Failed to create cache directory")?;

		Ok(self)
	}
}
