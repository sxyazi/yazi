use std::{io, path::{Path, PathBuf}};

use crate::cha::Cha;

pub struct Sftp;

impl Sftp {
	pub fn cache<P>(_: P) -> Option<PathBuf>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn canonicalize<P>(path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn copy<P, Q>(from: P, to: Q, cha: Cha) -> io::Result<u64>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		todo!()
	}

	pub async fn create<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn create_dir<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn create_dir_all<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn hard_link<P, Q>(original: P, link: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		todo!()
	}

	pub async fn metadata<P>(path: P) -> io::Result<std::fs::Metadata>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn open<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn read<P>(path: P) -> io::Result<Vec<u8>>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn read_dir<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub fn read_dir_sync<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn read_link<P>(path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn read_to_string<P>(path: P) -> io::Result<String>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn remove_dir<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn remove_dir_all<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn remove_file<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn rename<P, Q>(from: P, to: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		todo!()
	}

	pub async fn symlink<P, Q, F>(original: P, link: Q, _is_dir: F) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
		F: AsyncFnOnce() -> io::Result<bool>,
	{
		todo!()
	}

	pub async fn symlink_dir<P, Q>(original: P, link: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		todo!()
	}

	pub async fn symlink_file<P, Q>(original: P, link: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		todo!()
	}

	pub async fn symlink_metadata<P>(path: P) -> io::Result<std::fs::Metadata>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub fn symlink_metadata_sync<P>(path: P) -> io::Result<std::fs::Metadata>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn trash<P>(path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	pub async fn write<P, C>(path: P, contents: C) -> io::Result<()>
	where
		P: AsRef<Path>,
		C: AsRef<[u8]>,
	{
		todo!()
	}
}
