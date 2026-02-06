use anyhow::{Result, bail};
use serde::Deserialize;
use yazi_codegen::DeserializeOver2;

#[derive(Debug, Deserialize, DeserializeOver2)]
pub struct Tasks {
	pub file_workers:    u8,
	pub plugin_workers:  u8,
	pub fetch_workers:   u8,
	pub preload_workers: u8,
	pub process_workers: u8,

	pub bizarre_retry: u8,

	pub image_alloc: u32,
	pub image_bound: [u16; 2],

	pub suppress_preload: bool,
}

impl Tasks {
	pub(crate) fn reshape(self) -> Result<Self> {
		if self.file_workers < 1 {
			bail!("[tasks].file_workers must be at least 1.");
		} else if self.plugin_workers < 1 {
			bail!("[tasks].plugin_workers must be at least 1.");
		} else if self.fetch_workers < 1 {
			bail!("[tasks].fetch_workers must be at least 1.");
		} else if self.preload_workers < 1 {
			bail!("[tasks].preload_workers must be at least 1.");
		} else if self.process_workers < 1 {
			bail!("[tasks].process_workers must be at least 1.");
		} else if self.bizarre_retry < 1 {
			bail!("[tasks].bizarre_retry must be at least 1.");
		}
		Ok(self)
	}
}
