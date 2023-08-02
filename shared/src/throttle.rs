use std::{fmt::Debug, mem, sync::atomic::{AtomicU64, AtomicUsize, Ordering}, time::{self, Duration, SystemTime}};

use parking_lot::Mutex;

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
			last: AtomicU64::new(
				SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_millis() as u64
					- interval.as_millis() as u64,
			),
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
		let now = SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_millis() as u64;
		if now > self.interval.as_millis() as u64 + last {
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
		let mut buf = mem::replace(&mut *self.buf.lock(), Vec::new());
		buf.push(data);
		f(buf)
	}
}
