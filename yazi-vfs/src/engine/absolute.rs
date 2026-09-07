use yazi_fs::CWD;
use yazi_shared::url::{UrlCow, UrlLike};

pub(crate) fn try_absolute_impl<'a, U>(url: U) -> Option<UrlCow<'a>>
where
	U: Into<UrlCow<'a>>,
{
	let url = url.into();

	if url.is_absolute() {
		Some(url)
	} else if url.is_view() {
		let (uri, urn) = (url.uri(), url.urn());
		let abs: UrlCow = super::try_absolute(url.base().physical())?
			.into_owned()
			.into_view(url.auth().clone(), url.auth().view.data().cloned()?)
			.ok()?
			.try_join(uri)
			.ok()?
			.into();
		abs.with_ports(uri.components().count(), urn.components().count()).ok()
	} else if let cwd = CWD.load()
		&& cwd.auth().covariant(url.auth())
	{
		Some(cwd.try_join(url.loc()).ok()?.into())
	} else {
		None
	}
}
