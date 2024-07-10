use std::str::FromStr;

use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Tasks {
	#[validate(range(min = 1, message = "Cannot be less than 1"))]
	pub micro_workers: u8,
	#[validate(range(min = 1, message = "Cannot be less than 1"))]
	pub macro_workers: u8,
	#[validate(range(min = 1, message = "Cannot be less than 1"))]
	pub bizarre_retry: u8,

	pub image_alloc: u32,
	pub image_bound: [u16; 2],

	pub suppress_preload: bool,
}

impl FromStr for Tasks {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		#[derive(Deserialize)]
		struct Outer {
			tasks: Tasks,
		}

		let tasks = toml::from_str::<Outer>(s)?.tasks;
		tasks.validate()?;

		Ok(tasks)
	}
}
