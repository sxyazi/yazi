use std::io;

use tokio::{select, sync::mpsc::{self, UnboundedReceiver}};
use yazi_fs::{File, Files, FilesOp, cha::Cha, mounts::PARTITIONS, provider::{DirReader, FileHolder}};
use yazi_shared::url::UrlBuf;

use crate::{VfsCha, VfsFile, VfsFilesOp, provider::{self, DirEntry}};

pub trait VfsFiles {
	fn from_dir(dir: &UrlBuf) -> impl Future<Output = io::Result<UnboundedReceiver<File>>>;

	fn from_dir_bulk(dir: &UrlBuf) -> impl Future<Output = io::Result<Vec<File>>>;

	fn assert_stale(dir: &UrlBuf, cha: Cha) -> impl Future<Output = Option<Cha>>;
}

impl VfsFiles for Files {
	async fn from_dir(dir: &UrlBuf) -> std::io::Result<UnboundedReceiver<File>> {
		let mut it = provider::read_dir(dir).await?;
		let (tx, rx) = mpsc::unbounded_channel();

		tokio::spawn(async move {
			while let Ok(Some(ent)) = it.next().await {
				select! {
					_ = tx.closed() => break,
					result = ent.metadata() => {
						let url = ent.url();
						_ = tx.send(match result {
							Ok(cha) => File::from_follow(url, cha).await,
							Err(_) => File::from_dummy(url, ent.file_type().await.ok())
						});
					}
				}
			}
		});
		Ok(rx)
	}

	async fn from_dir_bulk(dir: &UrlBuf) -> std::io::Result<Vec<File>> {
		let mut it = provider::read_dir(dir).await?;
		let mut entries = Vec::new();
		while let Ok(Some(entry)) = it.next().await {
			entries.push(entry);
		}

		let (first, rest) = entries.split_at(entries.len() / 3);
		let (second, third) = rest.split_at(entries.len() / 3);
		async fn go(entries: &[DirEntry]) -> Vec<File> {
			let mut files = Vec::with_capacity(entries.len());
			for ent in entries {
				let url = ent.url();
				files.push(match ent.metadata().await {
					Ok(cha) => File::from_follow(url, cha).await,
					Err(_) => File::from_dummy(url, ent.file_type().await.ok()),
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

	async fn assert_stale(dir: &UrlBuf, cha: Cha) -> Option<Cha> {
		use std::io::ErrorKind;
		match Cha::from_url(dir).await {
			Ok(c) if !c.is_dir() => FilesOp::issue_error(dir, ErrorKind::NotADirectory).await,
			Ok(c) if c.hits(cha) && PARTITIONS.read().heuristic(cha) => {}
			Ok(c) => return Some(c),
			Err(e) => FilesOp::issue_error(dir, e).await,
		}
		None
	}
}
