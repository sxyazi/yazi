use serde::Deserialize;
use validator::Validate;

use crate::{validation::check_validation, MERGED_YAZI};

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
}

impl Default for Tasks {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			tasks: Tasks,
		}

		let tasks = toml::from_str::<Outer>(&MERGED_YAZI).unwrap().tasks;
		check_validation(tasks.validate());

		tasks
	}
}
