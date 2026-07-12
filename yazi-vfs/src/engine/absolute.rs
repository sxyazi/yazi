use yazi_fs::CWD;
use yazi_shared::url::{UrlCow, UrlLike};

pub(crate) fn try_absolute_impl<'a, U>(url: U) -> Option<UrlCow<'a>>
where
	U: Into<UrlCow<'a>>,
{
	let url = url.into();
	if url.is_absolute() {
		Some(url)
	} else if let cwd = CWD.load()
		&& cwd.auth().covariant(url.auth())
	{
		Some(cwd.try_join(url.loc()).ok()?.into())
	} else {
		None
	}
}
