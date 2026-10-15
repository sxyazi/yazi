use std::{io, path::{Path, PathBuf}};

use tokio::{sync::mpsc, task};

use super::Progress;
use crate::engine::{Attrs, Transmit};

pub struct Copier {
	pub(super) from:  PathBuf,
	pub(super) to:    PathBuf,
	pub(super) attrs: Attrs,

	pub(super) tx:       mpsc::Sender<io::Result<u64>>,
	rx:                  Option<mpsc::Receiver<io::Result<u64>>>,
	pub(super) progress: Progress,

	#[cfg(target_os = "macos")]
	pub(super) state:   super::Copystate,
	#[cfg(target_os = "macos")]
	pub(super) error:   Option<io::Error>,
	#[cfg(target_os = "macos")]
	pub(super) retries: u8,
}

impl Copier {
	pub(crate) fn new(from: &Path, to: &Path, attrs: Attrs) -> io::Result<Self> {
		let (tx, rx) = mpsc::channel(20);

		Ok(Self {
			from: from.to_owned(),
			to: to.to_owned(),
			attrs,
			tx,
			rx: Some(rx),

			progress: Progress::default(),

			#[cfg(target_os = "macos")]
			state: super::Copystate::new()?,
			#[cfg(target_os = "macos")]
			error: None,
			#[cfg(target_os = "macos")]
			retries: 0,
		})
	}

	pub(crate) fn spawn(mut self) -> Transmit {
		let rx = self.rx.take().unwrap();
		let tx = self.tx.clone();

		tokio::spawn(async move {
			let result = task::spawn_blocking(move || {
				loop {
					self.check()?;

					match self.copy() {
						Err(e) if self.retry(&e) => self.reset()?,
						result => break result?,
					}
				}

				Ok(self.progress.remaining())
			})
			.await
			.map_err(io::Error::from)
			.flatten();

			if let Ok(remaining) = &result
				&& *remaining > 0
			{
				tx.send(Ok(*remaining)).await.ok();
			}

			tx.send(result.map(|_| 0)).await.ok();
		});

		Transmit::new(rx)
	}

	pub(super) fn check(&self) -> io::Result<()> {
		if self.tx.is_closed() {
			Err(io::Error::new(io::ErrorKind::Interrupted, "Copy cancelled"))
		} else {
			Ok(())
		}
	}
}
