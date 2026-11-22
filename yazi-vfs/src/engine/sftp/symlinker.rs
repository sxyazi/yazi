use std::{borrow::Cow, io::{self, ErrorKind}};

use typed_path::{UnixPath, UnixPathBuf};
use yazi_fs::symlinker::Symlinker;
use yazi_macro::ok_or_not_found;
use yazi_sftp::Operator;
use yazi_shim::path::UnixPathExt;

use super::{Sftp, StatMode};

impl Sftp<'_> {
	pub(super) async fn symlink(&self, original: &UnixPath, force: bool) -> io::Result<()> {
		let op = self.op().await?;

		let result = op.symlink(original, self.path).await;
		if !force {
			return Ok(op.map_exists(self.path, result).await?);
		}

		let Err(failure) = result else { return Ok(()) };
		if !failure.maybe_exists() {
			return Err(failure.into());
		}

		let (link, target) = self.resolve_symlink(&op, original).await?;
		let attrs = ok_or_not_found!(op.lstat(&link).await, return Err(failure.into()));
		if StatMode::try_from(&attrs)?.0.is_dir() {
			return Err(io::Error::new(ErrorKind::IsADirectory, "Cannot overwrite a directory"));
		} else if ok_or_not_found!(op.realpath(&*target).await.map(|p| p == link)) {
			return Err(io::Error::new(ErrorKind::InvalidInput, "Cannot symlink a file onto itself"));
		}

		let tmp = link.with_file_name(Symlinker::temporary_name());
		op.symlink(original, &tmp).await?;

		match op.rename_posix(&tmp, &link).await {
			Ok(()) => Ok(()),
			Err(e) => {
				let _ = op.remove(&tmp).await;
				Err(e.into())
			}
		}
	}

	async fn resolve_symlink<'a>(
		&self,
		op: &Operator,
		original: &'a UnixPath,
	) -> io::Result<(UnixPathBuf, Cow<'a, UnixPath>)> {
		let (parent, name) = self.path.pair().ok_or(ErrorKind::InvalidInput)?;
		let parent = op.realpath(parent.non_empty().unwrap_or(UnixPath::new("."))).await?;

		let link = parent.join(name);
		let target = if original.is_absolute() {
			Cow::Borrowed(original)
		} else {
			Cow::Owned(parent.join(original))
		};

		Ok((link, target))
	}
}
