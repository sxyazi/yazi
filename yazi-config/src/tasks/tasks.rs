use std::num::NonZeroU8;

use serde::Deserialize;
use yazi_codegen::{DeserializeOver, DeserializeOver2};

#[derive(Debug, Deserialize, DeserializeOver, DeserializeOver2)]
pub struct Tasks {
	pub file_workers:    NonZeroU8,
	pub plugin_workers:  NonZeroU8,
	pub fetch_workers:   NonZeroU8,
	pub preload_workers: NonZeroU8,
	pub process_workers: NonZeroU8,

	pub bizarre_retry: NonZeroU8,

	pub image_alloc: u32,
	pub image_bound: [u16; 2],

	pub suppress_preload: bool,
}
