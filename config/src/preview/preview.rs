use std::{path::{Path, PathBuf}, time::{self, SystemTime}};

use md5::{Digest, Md5};
use serde::Deserialize;
use shared::absolute_path;

use super::PreviewAdaptor;
use crate::{xdg::Xdg, MERGED_YAZI};

#[derive(Debug)]
pub struct Preview {
	pub adaptor: PreviewAdaptor,

	pub tab_size:   u32,
	pub max_width:  u32,
	pub max_height: u32,

	pub cache_dir: PathBuf,
}

impl Default for Preview {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			preview: Shadow,
		}
		#[derive(Deserialize)]
		struct Shadow {
			pub tab_size:   u32,
			pub max_width:  u32,
			pub max_height: u32,

			pub cache_dir: Option<String>,
		}

		let preview = toml::from_str::<Outer>(&MERGED_YAZI).unwrap().preview;

		let cache_dir =
			preview.cache_dir.filter(|p| !p.is_empty()).map_or_else(Xdg::cache_dir, absolute_path);

		Preview {
			adaptor: Default::default(),

			tab_size: preview.tab_size,
			max_width: preview.max_width,
			max_height: preview.max_height,

			cache_dir,
		}
	}
}

impl Preview {
	#[inline]
	pub fn cache(&self, path: &Path, skip: usize) -> PathBuf {
		self
			.cache_dir
			.join(format!("{:x}", Md5::new_with_prefix(format!("{:?}///{}", path, skip)).finalize()))
	}

	#[inline]
	pub fn tmpfile(&self, prefix: &str) -> PathBuf {
		let nanos = SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_nanos();
		self.cache_dir.join(format!("{prefix}-{}", nanos / 1000))
	}
}
