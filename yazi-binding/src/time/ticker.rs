use std::time::{Duration, Instant};

use mlua::{MetaMethod, UserData, UserDataMethods};

pub struct Ticker {
	interval: Duration,
	last:     Instant,
}

impl Ticker {
	pub fn new(interval: Duration) -> Self { Self { interval, last: Instant::now() } }

	pub fn tick(&mut self) -> bool {
		if self.last.elapsed() >= self.interval {
			self.last = Instant::now();
			true
		} else {
			false
		}
	}
}

impl UserData for Ticker {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method_mut(MetaMethod::Call, |_, me, ()| Ok(me.tick()));
	}
}
