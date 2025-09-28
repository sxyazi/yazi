use std::io;

use yazi_fs::FilesOp;
use yazi_shared::url::UrlBuf;

use crate::maybe_exists;

pub trait VfsFilesOp {
	fn issue_error(cwd: &UrlBuf, kind: io::ErrorKind) -> impl Future<Output = ()>;
}

impl VfsFilesOp for FilesOp {
	async fn issue_error(cwd: &UrlBuf, kind: std::io::ErrorKind) {
		use std::io::ErrorKind;
		if kind != ErrorKind::NotFound {
			Self::IOErr(cwd.clone(), kind).emit();
		} else if maybe_exists(cwd).await {
			Self::IOErr(cwd.clone(), kind).emit();
		} else if let Some((p, n)) = cwd.pair() {
			Self::Deleting(p.into(), [n.into()].into()).emit();
		}
	}
}
