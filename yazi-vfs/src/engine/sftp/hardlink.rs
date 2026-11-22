use std::io::{self, ErrorKind};

use typed_path::{UnixPath, UnixPathBuf};
use yazi_fs::symlinker::Symlinker;
use yazi_macro::ok_or_not_found;
use yazi_sftp::Operator;

use super::{Sftp, StatMode};

impl Sftp<'_> {
	pub(super) async fn hardlink_imp(
		&self,
		to: &UnixPath,
		follow: bool,
		force: bool,
	) -> io::Result<()> {
		let op = self.op().await?;
		let resolved = if follow { Some(op.realpath(self.path).await?) } else { None };

		let result = op.hardlink(resolved.as_deref().unwrap_or(self.path), to).await;
		if !force {
			return Ok(op.map_exists(to, result).await?);
		}

		let Err(failure) = result else { return Ok(()) };
		if !failure.maybe_exists() {
			return Err(failure.into());
		}

		let to = Self::resolve_entry(&op, to).await?;
		let attrs = ok_or_not_found!(op.lstat(&to).await, return Err(failure.into()));
		if StatMode::try_from(&attrs)?.0.is_dir() {
			return Err(io::Error::new(ErrorKind::IsADirectory, "Cannot overwrite a directory"));
		}

		let from = if let Some(p) = resolved { p } else { Self::resolve_entry(&op, self.path).await? };
		if from == to {
			return Err(io::Error::new(ErrorKind::InvalidInput, "Cannot hardlink a file onto itself"));
		}

		let tmp = Self::temporary_hardlink(&op, &from, &to).await?;
		let result = op.rename_posix(&tmp, &to).await;
		_ = op.remove(&tmp).await; // Renaming two hardlinks to the same file can leave both names in place.
		Ok(result?)
	}

	async fn temporary_hardlink(
		op: &Operator,
		from: &UnixPath,
		to: &UnixPath,
	) -> io::Result<UnixPathBuf> {
		let mut retries = 7;
		loop {
			let tmp = to.with_file_name(Symlinker::temporary_name());
			let result = op.hardlink(from, &tmp).await;
			match op.map_exists(&tmp, result).await.map_err(io::Error::from) {
				Ok(()) => return Ok(tmp),
				Err(e) if e.kind() == ErrorKind::AlreadyExists && retries > 0 => retries -= 1,
				Err(e) => return Err(e),
			}
		}
	}
}
