use std::time::{SystemTime, UNIX_EPOCH};

pub struct LcgRng {
	seed: u64,
}

impl LcgRng {
	const A: u64 = 6364136223846793005;
	const C: u64 = 1;
	const M: u64 = u64::MAX;
}

impl Iterator for LcgRng {
	type Item = u64;

	fn next(&mut self) -> Option<Self::Item> {
		self.seed = Self::A.wrapping_mul(self.seed).wrapping_add(Self::C) % Self::M;
		Some(self.seed)
	}
}

impl Default for LcgRng {
	fn default() -> Self {
		let time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
		Self { seed: time.as_secs() ^ time.subsec_nanos() as u64 }
	}
}
