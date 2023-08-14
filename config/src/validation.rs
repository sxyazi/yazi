use std::{borrow::Cow, process};

use validator::{ValidationErrors, ValidationErrorsKind};

pub fn check_validation(res: Result<(), ValidationErrors>) {
	let Err(errors) = res else {
		return;
	};

	for (field, kind) in errors.into_errors() {
		match kind {
			ValidationErrorsKind::Struct(errors) => check_validation(Err(*errors)),
			ValidationErrorsKind::List(errors) => {
				for (i, errors) in errors {
					print!("Config `{field}[{i}]` format error: ");
					check_validation(Err(*errors));
					println!();
				}
			}
			ValidationErrorsKind::Field(error) => {
				for e in error {
					println!(
						"Config `{field}` format error: {}\n",
						e.message.unwrap_or(Cow::Borrowed("unknown error"))
					);
				}
			}
		}
	}
	process::exit(1);
}
