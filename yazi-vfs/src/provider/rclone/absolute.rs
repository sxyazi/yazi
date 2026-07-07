use yazi_fs::CWD;
use yazi_shared::url::{UrlCow, UrlLike};

pub fn try_absolute<'a, U>(url: U) -> Option<UrlCow<'a>>
where
	U: Into<UrlCow<'a>>,
{
	try_absolute_impl(url.into())
}

fn try_absolute_impl<'a>(url: UrlCow<'a>) -> Option<UrlCow<'a>> {
	if url.is_absolute() {
		Some(url)
	} else if let cwd = CWD.load()
		&& cwd.scheme().covariant(url.scheme())
	{
		Some(cwd.try_join(url.loc()).ok()?.into())
	} else {
		None
	}
}
