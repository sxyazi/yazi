use std::path::PathBuf;

use yazi_shared::url::UrlBuf;

use crate::path::{clean_url, expand_url};

pub fn sanitize_path(path: PathBuf) -> Option<PathBuf> {
	clean_url(yazi_fs::engine::local::try_absolute(expand_url(UrlBuf::from(path)))?)
		.into_loc()
		.into_os()
		.ok()
		.filter(|p| p.as_os_str().is_empty() || p.is_absolute())
}
