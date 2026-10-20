use std::{io, time::{Duration, Instant}};

use tokio::sync::mpsc;

use super::Delta;

pub struct Progress {
	pending: u64,
	updated: Instant,
}

impl Default for Progress {
	fn default() -> Self { Self { pending: 0, updated: Instant::now() } }
}

impl Progress {
	pub fn advance(&mut self, tx: &mpsc::Sender<io::Result<Delta>>) {
		self.pending += 1;

		if self.updated.elapsed() >= Duration::from_millis(250)
			&& tx.try_send(Ok(self.pending.into())).is_ok()
		{
			self.pending = 0;
			self.updated = Instant::now();
		}
	}

	pub fn remaining(&self) -> u64 { self.pending }
}
