use std::io;

use tokio::{select, sync::mpsc::{self, UnboundedReceiver}};
use yazi_fs::{Entries, engine::{DirReader, FileHolder}, file::File, mounts::PARTITIONS};
use yazi_shared::url::UrlBuf;

use crate::engine::{self, DirEntry};

pub trait VfsEntries {
	fn from_dir(dir: &UrlBuf) -> impl Future<Output = io::Result<UnboundedReceiver<File>>>;

	fn from_dir_bulk(dir: &UrlBuf) -> impl Future<Output = io::Result<Vec<File>>>;

	fn revalidate(file: &File) -> impl Future<Output = io::Result<Option<File>>>;
}

impl VfsEntries for Entries {
	async fn from_dir(dir: &UrlBuf) -> std::io::Result<UnboundedReceiver<File>> {
		let mut it = engine::read_dir(dir).await?;
		let (tx, rx) = mpsc::unbounded_channel();

		tokio::spawn(async move {
			while let Ok(Some(ent)) = it.next().await {
				select! {
					_ = tx.closed() => break,
					result = ent.file() => {
						_ = tx.send(match result {
							Ok(file) => file,
							Err(_) => File::from_dummy(ent.url(), ent.file_type().await.ok()),
						});
					}
				}
			}
		});
		Ok(rx)
	}

	async fn from_dir_bulk(dir: &UrlBuf) -> std::io::Result<Vec<File>> {
		let mut it = engine::read_dir(dir).await?;
		let mut entries = Vec::new();
		while let Ok(Some(entry)) = it.next().await {
			entries.push(entry);
		}

		let (first, rest) = entries.split_at(entries.len() / 3);
		let (second, third) = rest.split_at(entries.len() / 3);
		async fn go(entries: &[DirEntry]) -> Vec<File> {
			let mut files = Vec::with_capacity(entries.len());
			for ent in entries {
				files.push(match ent.file().await {
					Ok(file) => file,
					Err(_) => File::from_dummy(ent.url(), ent.file_type().await.ok()),
				});
			}
			files
		}

		Ok(
			futures::future::join_all([go(first), go(second), go(third)])
				.await
				.into_iter()
				.flatten()
				.collect(),
		)
	}

	async fn revalidate(old: &File) -> io::Result<Option<File>> {
		match engine::revalidate(old).await? {
			Some(new) if new.url != old.url => {
				Err(io::Error::new(io::ErrorKind::InvalidData, "revalidated file URL changed"))
			}
			Some(new) if !new.is_dir() => Err(io::ErrorKind::NotADirectory.into()),
			Some(new) => Ok(Some(new)),
			None if PARTITIONS.read().timeless(old.cha) => Ok(Some(old.clone())),
			None => Ok(None),
		}
	}
}
