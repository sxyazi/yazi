use std::{io, path::{Path, PathBuf}};

use tokio::{sync::mpsc, task};

use super::{Delta, Progress, Removal};

pub struct Remover {
	pub(super) path: PathBuf,

	pub(super) tx:       mpsc::Sender<io::Result<Delta>>,
	pub(super) progress: Progress,
}

impl Remover {
	pub fn spawn(path: &Path) -> Removal {
		let (tx, rx) = mpsc::channel(20);

		let mut me =
			Self { path: path.to_owned(), tx: tx.clone(), progress: Progress::default() };

		let handle = tokio::spawn(async move {
			let (result, remaining) = task::spawn_blocking(move || {
				let result = me.check().and_then(|()| me.remove());
				(result, me.progress.remaining())
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
