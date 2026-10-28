use std::io;

use tokio::{select, sync::mpsc};
use yazi_fs::{engine::{DirReader, FileHolder}, remover::{Delta, Progress, Removal}, stat::Stat};
use yazi_macro::ok_or_not_found;
use yazi_shared::url::UrlBuf;

use crate::engine;

pub(crate) struct Remover {
	tx:       mpsc::Sender<io::Result<Delta>>,
	progress: Progress,
}

impl Remover {
	pub(crate) fn spawn(target: UrlBuf) -> Removal {
		let (tx, rx) = mpsc::channel(20);

		let mut me = Self { tx: tx.clone(), progress: Progress::default() };

		let handle = tokio::spawn(async move {
			let result = select! {
				biased;
				_ = tx.closed() => return,
				result = me.remove_root(target) => result,
			};

			let remaining = me.progress.remaining();
			if remaining > 0 {
				tx.send(Ok(remaining.into())).await.ok();
			}
			if let Err(e) = result {
				tx.send(Err(e)).await.ok();
			}
		});

		Removal::new(rx, handle)
	}

	async fn remove_root(&mut self, target: UrlBuf) -> io::Result<()> {
		let stat = ok_or_not_found!(engine::symlink_metadata(&target).await, return Ok(()));
		if !stat.is_dir() && !stat.is_indirect() {
			return Err(io::ErrorKind::NotADirectory.into());
		}

		self.remove_children(target, stat).await
	}

	async fn remove_children(&mut self, target: UrlBuf, stat: Stat) -> io::Result<()> {
		if stat.is_dir() && !stat.is_indirect() {
			let mut it = ok_or_not_found!(engine::read_dir(&target).await, return Ok(()));
			while let Some(dent) = it.next().await? {
				let stat = ok_or_not_found!(dent.metadata().await, continue);
				() = ok_or_not_found!(Box::pin(self.remove_children(dent.url(), stat)).await);
			}
		}

		let result = if stat.is_dir() {
			engine::remove_dir(&target).await
		} else {
			engine::remove_file(&target).await
		};

		ok_or_not_found!(result, return Ok(()));
		self.progress.advance(&self.tx);

		Ok(())
	}
}
