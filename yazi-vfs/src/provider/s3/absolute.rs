use yazi_fs::CWD;
use yazi_shared::{loc::LocBuf, pool::InternStr, url::{AsUrl, Url, UrlBuf, UrlCow, UrlLike}};

pub fn try_absolute<'a, U>(url: U) -> Option<UrlCow<'a>>
where
	U: Into<UrlCow<'a>>,
{
	let url = url.into();
	if url.is_absolute() {
		Some(url)
	} else if let Url::S3 { domain, .. } = url.as_url() {
		let raw = url.loc().to_string_lossy();
		let raw = raw.trim_start_matches('/');
		let absolute = if raw.is_empty() { "/".to_owned() } else { format!("/{raw}") };
		Some(
			UrlBuf::S3 {
				loc:    LocBuf::<typed_path::UnixPathBuf>::zeroed(typed_path::UnixPathBuf::from(absolute)),
				domain: domain.intern(),
			}
			.into(),
		)
	} else if let cwd = CWD.load()
		&& cwd.scheme().covariant(url.scheme())
	{
		Some(cwd.try_join(url.loc()).ok()?.into())
	} else {
		None
	}
}
