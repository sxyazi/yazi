use serde::Deserialize;
use validator::Validate;

use crate::{validation::check_validation, MERGED_YAZI};

#[derive(Debug, Deserialize, Validate)]
pub struct Tasks {
	#[validate(range(min = 3, message = "Cannot be less than 3"))]
	pub micro_workers: u8,
	#[validate(range(min = 5, message = "Cannot be less than 5"))]
	pub macro_workers: u8,
	#[validate(range(min = 3, message = "Cannot be less than 3"))]
	pub bizarre_retry: u8,
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
