use std::io;

use tokio::sync::mpsc;

pub struct Transmit {
	rx:      Option<mpsc::Receiver<io::Result<u64>>>,
	pending: Option<io::Result<u64>>,
}

impl From<mpsc::Receiver<io::Result<u64>>> for Transmit {
	fn from(rx: mpsc::Receiver<io::Result<u64>>) -> Self { Self::new(rx) }
}

impl Transmit {
	pub fn new(rx: mpsc::Receiver<io::Result<u64>>) -> Self { Self { rx: Some(rx), pending: None } }

	pub fn unsupported() -> Self { Self { rx: None, pending: None } }

	pub async fn is_supported(&mut self) -> bool {
		let Some(rx) = self.rx.as_mut() else { return false };

		if self.pending.is_none() {
			self.pending = rx.recv().await;
		}

		if matches!(&self.pending, Some(Err(e)) if e.kind() == io::ErrorKind::CrossesDevices) {
			(self.rx, self.pending) = (None, None);
			false
		} else {
			true
		}
	}

	pub async fn recv(&mut self) -> Option<io::Result<u64>> {
		if let Some(value) = self.pending.take() {
			return Some(value);
		}

		self.rx.as_mut()?.recv().await
	}

	pub async fn total(mut self) -> io::Result<u64> {
		let mut total = 0;
		while let Some(n) = self.recv().await {
			total += n?;
		}
		Ok(total)
	}
}
