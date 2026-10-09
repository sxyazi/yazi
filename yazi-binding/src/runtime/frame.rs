use std::ops::Deref;

use super::RuntimeSeed;

#[derive(Debug)]
pub(super) struct RuntimeFrame {
	pub(super) seed:     RuntimeSeed,
	pub(super) blocking: bool,
}

impl Deref for RuntimeFrame {
	type Target = RuntimeSeed;

	fn deref(&self) -> &Self::Target { &self.seed }
}
