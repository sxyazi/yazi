use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct Status {
	pub id:       u32,
	pub code:     StatusCode,
	pub message:  String,
	pub language: String,
}

impl From<Status> for Result<(), Error> {
	fn from(status: Status) -> Self {
		if status.is_ok() { Ok(()) } else { Err(Error::Status(status)) }
	}
}

impl Status {
	pub fn len(&self) -> usize {
		size_of_val(&self.id)
			+ size_of_val(&(self.code as u32))
			+ 4 + self.message.len()
			+ 4 + self.language.len()
	}

	pub fn is_ok(&self) -> bool { self.code == StatusCode::Ok }

	pub fn is_eof(&self) -> bool { self.code == StatusCode::Eof }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum StatusCode {
	Ok               = 0,
	Eof              = 1,
	NoSuchFile       = 2,
	PermissionDenied = 3,
	Failure          = 4,
	BadMessage       = 5,
	NoConnection     = 6,
	ConnectionLost   = 7,
	OpUnsupported    = 8,
}
