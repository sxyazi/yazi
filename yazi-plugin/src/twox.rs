use std::hash::Hasher;

pub struct Twox128(twox_hash::XxHash3_128);

impl Default for Twox128 {
	fn default() -> Self { Self(twox_hash::XxHash3_128::new()) }
}

impl Twox128 {
	pub fn finish_128(self) -> u128 { self.0.finish_128() }
}

impl Hasher for Twox128 {
	fn write(&mut self, bytes: &[u8]) { self.0.write(bytes) }

	fn finish(&self) -> u64 { unreachable!() }
}
