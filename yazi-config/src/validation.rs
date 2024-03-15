use std::{borrow::Cow, process};

use validator::{ValidationErrors, ValidationErrorsKind};

pub fn check_validation(res: Result<(), ValidationErrors>) {
	let Err(errors) = res else { return };

	for (field, kind) in errors.into_errors() {
		match kind {
			ValidationErrorsKind::Struct(errors) => check_validation(Err(*errors)),
			ValidationErrorsKind::List(errors) => {
				for (i, errors) in errors {
					eprint!("Config `{field}[{i}]` format error: ");
					check_validation(Err(*errors));
					eprintln!();
				}
			}
			ValidationErrorsKind::Field(error) => {
				for e in error {
					eprintln!(
						"Config `{field}` format error: {}\n",
						e.message.unwrap_or(Cow::Borrowed("unknown error"))
					);
				}
			}
		}
	}
	process::exit(1);
}
