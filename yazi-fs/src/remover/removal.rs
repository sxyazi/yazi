use std::io;

use tokio::{sync::mpsc, task::JoinHandle};

use super::Delta;

pub struct Removal {
	rx:     Option<mpsc::Receiver<io::Result<Delta>>>,
	handle: JoinHandle<()>,
}

impl Removal {
	pub fn new(rx: mpsc::Receiver<io::Result<Delta>>, handle: JoinHandle<()>) -> Self {
		Self { rx: Some(rx), handle }
	}

	pub async fn recv(&mut self) -> Option<io::Result<Delta>> {
		if let Some(result) = self.rx.as_mut()?.recv().await {
			if result.is_err() {
				self.rx = None;
			}
			return Some(result);
		}

		let result = (&mut self.handle).await;
		self.rx = None;
		result.err().map(|e| Err(e.into()))
	}

	pub async fn total(mut self) -> io::Result<Delta> {
		let mut total = Delta::default();
		while let Some(delta) = self.recv().await {
			total = total.try_add(delta?).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
		}
		Ok(total)
	}

	pub async fn finish(self) -> io::Result<()> {
		let total = self.total().await?;
		if total.failed == 0 {
			Ok(())
		} else {
			Err(io::Error::other(format!("Failed to remove {} entries", total.failed)))
		}
	}
}
