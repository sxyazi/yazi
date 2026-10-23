use std::{mem, time::{Duration, Instant}};

use parking_lot::Mutex;

#[derive(Debug)]
pub struct Throttle<T> {
	interval: Duration,
	inner:    Mutex<(usize, Instant, Vec<T>)>,
}

impl<T> Throttle<T> {
	pub fn new(total: usize, interval: Duration) -> Self {
		Self { interval, inner: Mutex::new((total, Instant::now() - interval, Vec::new())) }
	}

	pub fn done<F>(&self, data: T, f: F)
	where
		F: FnOnce(Vec<T>),
	{
		let mut inner = self.inner.lock();
		let (total, last, buf) = &mut *inner;

		*total -= 1;
		buf.push(data);

		if *total != 0 && last.elapsed() <= self.interval {
			return;
		}

		*last = Instant::now();
		let buf = mem::take(buf);
		drop(inner);

		f(buf)
	}
}
