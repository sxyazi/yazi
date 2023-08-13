use serde::Deserialize;

use crate::MERGED_YAZI;

#[derive(Debug, Deserialize)]
pub struct Tasks {
	pub micro_workers: u8,
	pub macro_workers: u8,
	pub bizarre_retry: u8,
}

impl Default for Tasks {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			tasks: Tasks,
		}

		let tasks = toml::from_str::<Outer>(&MERGED_YAZI).unwrap().tasks;
		if tasks.micro_workers <= 2 || tasks.macro_workers <= 2 {
			panic!("micro_workers, and macro_workers must be greater than 2");
		}

		tasks
	}
}
