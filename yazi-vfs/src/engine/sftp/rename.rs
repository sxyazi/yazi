use std::{hash::BuildHasher, io::{self, ErrorKind}};

use tokio::try_join;
use typed_path::{UnixPath, UnixPathBuf};
use yazi_macro::ok_or_not_found;
use yazi_sftp::Operator;

use super::{Sftp, StatMode};

impl Sftp<'_> {
	pub(super) async fn rename_fallback(
		op: &Operator,
		from: &UnixPath,
		to: &UnixPath,
	) -> io::Result<()> {
		let Err(failure) = op.rename(from, to).await else { return Ok(()) };
		if !failure.maybe_exists() {
			return Err(failure.into());
		}

		let attrs = ok_or_not_found!(op.lstat(to).await, return Err(failure.into()));
		let (from, to) = try_join!(Self::resolve_entry(op, from), Self::resolve_entry(op, to))?;
		if from == to {
			return Ok(());
		} else if StatMode::try_from(&attrs)?.0.is_dir() {
			return Err(io::Error::new(ErrorKind::IsADirectory, "Cannot overwrite a directory"));
		}

		let backup = Self::backup_entry(op, &to).await?;
		match op.rename(&from, &to).await {
			Ok(()) => {
				op.remove(&backup).await.ok();
				Ok(())
			}
			Err(e) => {
				op.rename(&backup, &to).await.ok();
				Err(e.into())
			}
		}
	}

	async fn backup_entry(op: &Operator, path: &UnixPath) -> io::Result<UnixPathBuf> {
		let h = foldhash::fast::RandomState::default();
		let mut retries = 7;

		loop {
			let backup = path.with_file_name(format!(".yazirename-{:016x}", h.hash_one((path, retries))));
			let result = op.rename(path, &backup).await;
			match op.map_exists(&backup, result).await.map_err(io::Error::from) {
				Ok(()) => return Ok(backup),
				Err(e) if e.kind() == ErrorKind::AlreadyExists && retries > 0 => retries -= 1,
				Err(e) => {
					return Err(io::Error::new(
						e.kind(),
						format!("Cannot back up SFTP destination to {}: {e}", backup.display()),
					));
				}
			}
		}
	}
}
