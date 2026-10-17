use std::{io, time::{Duration, Instant}};

use tokio::sync::mpsc;

pub(super) struct Progress {
	written:  u64,
	reported: u64,
	updated:  Instant,
	scale:    Option<(u64, u64)>,
}

impl Default for Progress {
	fn default() -> Self { Self { written: 0, reported: 0, updated: Instant::now(), scale: None } }
}

impl Progress {
	#[cfg(any(target_os = "linux", target_os = "android"))]
	pub(super) fn written(&self) -> u64 { self.written }

	pub(super) fn remaining(&self) -> u64 { self.written.saturating_sub(self.reported) }

	#[cfg(all(unix, not(target_os = "macos")))]
	pub(super) fn advance(&mut self, bytes: usize, tx: &mpsc::Sender<io::Result<u64>>) {
		self.written += bytes as u64;
		self.report(tx);
	}

	pub(super) fn update(&mut self, bytes: u64) {
		// Fork copying counts stored bytes; map their completion ratio to the logical file size.
		self.written = match self.scale {
			Some((logical, stored)) if stored > 0 => {
				((bytes.min(stored) as u128 * logical as u128) / stored as u128) as u64
			}
			Some(_) => 0,
			None => bytes,
		};
	}

	pub(super) fn scale(&mut self, logical: u64, stored: u64) {
		self.scale = Some((logical, stored));
	}

	pub(super) fn unscale(&mut self) { self.scale = None; }

	pub(super) fn report(&mut self, tx: &mpsc::Sender<io::Result<u64>>) {
		if self.written > self.reported
			&& self.updated.elapsed() >= Duration::from_millis(200)
			&& tx.try_send(Ok(self.written - self.reported)).is_ok()
		{
			self.reported = self.written;
			self.updated = Instant::now();
		}
	}
}
