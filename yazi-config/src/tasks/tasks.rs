use std::str::FromStr;

use anyhow::Context;
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

		let outer = toml::from_str::<Outer>(s)
			.context("Failed to parse the [tasks] section in your yazi.toml")?;
		outer.tasks.validate()?;

		Ok(outer.tasks)
	}
}
