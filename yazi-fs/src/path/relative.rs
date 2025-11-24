use anyhow::{Result, bail};
use yazi_shared::{path::PathBufDyn, url::{UrlCow, UrlLike}};

pub fn url_relative_to<'a, 'b, U, V>(from: U, to: V) -> Result<UrlCow<'b>>
where
	U: Into<UrlCow<'a>>,
	V: Into<UrlCow<'b>>,
{
	url_relative_to_(from.into(), to.into())
}

fn url_relative_to_<'a>(from: UrlCow<'_>, to: UrlCow<'a>) -> Result<UrlCow<'a>> {
	use yazi_shared::url::Component::*;

	if from.is_absolute() != to.is_absolute() {
		return if to.is_absolute() {
			Ok(to)
		} else {
			bail!("Urls must be both absolute or both relative: {from:?} and {to:?}");
		};
	}

	if from.covariant(&to) {
		return UrlCow::try_from((
			to.scheme().zeroed().to_owned(),
			PathBufDyn::with_str(to.kind(), "."),
		));
	}

	let (mut f_it, mut t_it) = (from.components(), to.components());
	let (f_head, t_head) = loop {
		match (f_it.next(), t_it.next()) {
			(Some(Scheme(a)), Some(Scheme(b))) if a.covariant(b) => {}
			(Some(RootDir), Some(RootDir)) => {}
			(Some(Prefix(a)), Some(Prefix(b))) if a == b => {}
			(Some(Scheme(_) | Prefix(_) | RootDir), _) | (_, Some(Scheme(_) | Prefix(_) | RootDir)) => {
				return Ok(to);
			}
			(None, None) => break (None, None),
			(a, b) if a != b => break (a, b),
			_ => (),
		}
	};

	let dots = f_head.into_iter().chain(f_it).map(|_| ParentDir);
	let rest = t_head.into_iter().chain(t_it);

	let iter = dots.chain(rest).map(|c| c.downgrade().expect("path component from dot or normal"));
	let buf = PathBufDyn::from_components(to.kind(), iter)?;

	UrlCow::try_from((to.scheme().zeroed().to_owned(), buf))
}
