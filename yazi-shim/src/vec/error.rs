use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("index {index} is out of bounds for Vec of len {len}")]
pub struct IndexAtError {
	pub index: isize,
	pub len:   usize,
}
