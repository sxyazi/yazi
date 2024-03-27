use std::{fmt::Debug, mem, sync::atomic::{AtomicU64, AtomicUsize, Ordering}, time::Duration};

use parking_lot::Mutex;

use crate::timestamp_us;

#[derive(Debug)]
pub struct Throttle<T> {
	total:    AtomicUsize,
	interval: Duration,
	last:     AtomicU64,
	buf:      Mutex<Vec<T>>,
}

impl<T> Throttle<T> {
	pub fn new(total: usize, interval: Duration) -> Self {
		Self {
			total: AtomicUsize::new(total),
			interval,
			last: AtomicU64::new(timestamp_us() - interval.as_micros() as u64),
			buf: Default::default(),
		}
	}

	pub fn done<F>(&self, data: T, f: F)
	where
		F: FnOnce(Vec<T>),
	{
		let total = self.total.fetch_sub(1, Ordering::Relaxed);
		if total == 1 {
			return self.flush(data, f);
		}

		let last = self.last.load(Ordering::Relaxed);
		let now = timestamp_us();
		if now > self.interval.as_micros() as u64 + last {
			self.last.store(now, Ordering::Relaxed);
			return self.flush(data, f);
		}

		self.buf.lock().push(data);
	}

	#[inline]
	fn flush<F>(&self, data: T, f: F)
	where
		F: FnOnce(Vec<T>),
	{
		let mut buf = mem::take(&mut *self.buf.lock());
		buf.push(data);
		f(buf)
	}
}
