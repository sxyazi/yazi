use super::{Input, InputMode, InputOp};

#[derive(Default, PartialEq, Eq)]
pub struct InputSnap {
	value: String,

	op:    InputOp,
	start: Option<usize>,

	mode:   InputMode,
	offset: usize,
	cursor: usize,
}

impl From<&Input> for InputSnap {
	fn from(input: &Input) -> Self {
		Self {
			value: input.value.clone(),

			op:    input.op,
			start: input.start,

			mode:   input.mode,
			offset: input.offset,
			cursor: input.cursor,
		}
	}
}
