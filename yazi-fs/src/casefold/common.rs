use std::{io, path::{Path, PathBuf}};

use tokio::task;

pub struct Casefold;

impl Casefold {
	pub(crate) async fn casefold(path: impl AsRef<Path>) -> io::Result<PathBuf> {
		let path = path.as_ref();
		if path.as_os_str().as_encoded_bytes().contains(&0) {
			return Err(io::ErrorKind::InvalidInput.into());
		} else if path.file_name().is_none() {
			return Ok(path.to_owned());
		}

		let path = path.to_owned();
		task::spawn_blocking(move || Ok(path.with_file_name(Self::final_name(&path)?))).await?
	}

	pub async fn match_name_case(path: impl AsRef<Path>) -> io::Result<bool> {
		let path = path.as_ref();
		Ok(Self::casefold(path).await?.file_name() == path.file_name())
	}
}
