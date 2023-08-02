use std::time::Duration;

use tokio::{sync::mpsc::{self, UnboundedReceiver, UnboundedSender}, time::Instant};

pub struct DelayedBuffer<T> {
	buf:      Vec<T>,
	tx:       UnboundedSender<Vec<T>>,
	last:     Instant,
	interval: Duration,
}

impl<T> DelayedBuffer<T> {
	pub fn new(interval: Duration) -> (Self, UnboundedReceiver<Vec<T>>) {
		let (tx, rx) = mpsc::unbounded_channel();
		(Self { buf: Vec::new(), tx, last: Instant::now() - interval, interval }, rx)
	}

	pub fn push(&mut self, item: T) {
		self.buf.push(item);
		if self.last.elapsed() >= self.interval {
			self.last = Instant::now();
			self.tx.send(self.buf.drain(..).collect()).ok();
		}
	}

	pub fn flush(&mut self) {
		if !self.buf.is_empty() {
			self.tx.send(self.buf.drain(..).collect()).ok();
		}
	}
}

impl<T> Drop for DelayedBuffer<T> {
	fn drop(&mut self) {
		self.flush();
		self.tx.send(vec![]).ok();
	}
}
