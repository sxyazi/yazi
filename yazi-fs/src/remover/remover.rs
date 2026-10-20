use std::{io, path::{Path, PathBuf}};

use tokio::{sync::mpsc, task};

use super::{Delta, Progress, Removal};

pub struct Remover {
	pub(super) path: PathBuf,

	pub(super) tx:       mpsc::Sender<io::Result<Delta>>,
	rx:                  Option<mpsc::Receiver<io::Result<Delta>>>,
	pub(super) progress: Progress,
}

impl Remover {
	pub fn new(path: &Path) -> Self {
		let (tx, rx) = mpsc::channel(20);

		Self { path: path.to_owned(), tx, rx: Some(rx), progress: Progress::default() }
	}

	pub fn spawn(mut self) -> Removal {
		let rx = self.rx.take().unwrap();
		let tx = self.tx.clone();

		let handle = tokio::spawn(async move {
			let (result, remaining) = task::spawn_blocking(move || {
				let result = self.check().and_then(|()| self.remove());
				(result, self.progress.remaining())
			})
			.await
			.unwrap_or_else(|e| (Err(e.into()), 0));

			if remaining > 0 {
				tx.send(Ok(remaining.into())).await.ok();
			}
			if let Err(e) = result {
				tx.send(Err(e)).await.ok();
			}
		});

		Removal::new(rx, handle)
	}

	pub(super) fn check(&self) -> io::Result<()> {
		if self.tx.is_closed() {
			Err(io::Error::new(io::ErrorKind::Interrupted, "Removal cancelled"))
		} else {
			Ok(())
		}
	}
}
