use anyhow::{Result, bail};
use serde::Deserialize;
use yazi_codegen::DeserializeOver2;

#[derive(Debug, Deserialize, DeserializeOver2)]
pub struct Tasks {
	pub micro_workers: u8,
	pub macro_workers: u8,
	pub bizarre_retry: u8,

	pub image_alloc: u32,
	pub image_bound: [u16; 2],

	pub suppress_preload: bool,
}

impl Tasks {
	pub(crate) fn reshape(self) -> Result<Self> {
		if self.micro_workers < 1 {
			bail!("[tasks].micro_workers must be at least 1.");
		} else if self.macro_workers < 1 {
			bail!("[tasks].macro_workers must be at least 1.");
		} else if self.bizarre_retry < 1 {
			bail!("[tasks].bizarre_retry` must be at least 1.");
		}
		Ok(self)
	}
}
