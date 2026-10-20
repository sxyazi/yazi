use std::io;

use tokio::{sync::mpsc, task::JoinHandle};

/// Producers send work increments and errors; their task must finish normally for the operation to succeed.
/// Dropping the receiver cancels the operation.
pub struct Transit {
	rx:      Option<mpsc::Receiver<io::Result<u64>>>,
	handle:  Option<JoinHandle<()>>,
	pending: Option<io::Result<u64>>,
}

impl Transit {
	pub fn new(rx: mpsc::Receiver<io::Result<u64>>, handle: JoinHandle<()>) -> Self {
		Self { rx: Some(rx), handle: Some(handle), pending: None }
	}

	pub fn unsupported() -> Self { Self { rx: None, handle: None, pending: None } }

	pub async fn is_supported(&mut self) -> bool {
		if self.pending.is_none() {
			self.pending = self.recv().await;
		}

		if matches!(&self.pending, Some(Err(e)) if e.kind() == io::ErrorKind::CrossesDevices) {
			*self = Self::unsupported();
			false
		} else {
			self.handle.is_some()
		}
	}

	pub async fn recv(&mut self) -> Option<io::Result<u64>> {
		if let Some(value) = self.pending.take() {
			return Some(value);
		}

		if let Some(result) = self.rx.as_mut()?.recv().await {
			if result.is_err() {
				self.rx = None;
			}
			return Some(result);
		}

		let result = self.handle.as_mut()?.await;
		self.rx = None;
		result.err().map(|e| Err(e.into()))
	}

	pub async fn total(mut self) -> io::Result<u64> {
		let mut total = 0;
		while let Some(n) = self.recv().await {
			total += n?;
		}
		Ok(total)
	}
}
