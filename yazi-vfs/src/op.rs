use yazi_fs::FilesOp;
use yazi_shared::url::{UrlBuf, UrlLike};

use crate::maybe_exists;

pub trait VfsFilesOp {
	fn issue_error(cwd: &UrlBuf, kind: impl Into<yazi_fs::error::Error>) -> impl Future<Output = ()>;
}

impl VfsFilesOp for FilesOp {
	async fn issue_error(cwd: &UrlBuf, err: impl Into<yazi_fs::error::Error>) {
		let err = err.into();
		if err.kind() != std::io::ErrorKind::NotFound {
			Self::IOErr(cwd.clone(), err).emit();
		} else if maybe_exists(cwd).await {
			Self::IOErr(cwd.clone(), err).emit();
		} else if let Some((p, n)) = cwd.pair() {
			Self::Deleting(p.into(), [n.into()].into()).emit();
		}
	}
}
