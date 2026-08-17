use crate::EMULATOR;

#[must_use]
pub struct Deinit;

impl Drop for Deinit {
	fn drop(&mut self) { EMULATOR.stop(); }
}
