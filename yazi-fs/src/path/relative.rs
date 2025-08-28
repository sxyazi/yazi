use std::{borrow::Cow, path::{Path, PathBuf}};

use anyhow::{Result, bail};
use yazi_shared::{loc::LocBuf, url::{Url, UrlBuf, UrlCow}};

pub fn path_relative_to<'a>(
	from: impl AsRef<Path>,
	to: &'a impl AsRef<Path>,
) -> Result<Cow<'a, Path>> {
	Ok(match url_relative_to(Url::regular(&from).into(), Url::regular(to).into())? {
		UrlCow::Borrowed { loc, .. } => Cow::Borrowed(loc.as_path()),
		UrlCow::Owned { loc, .. } => Cow::Owned(loc.into_path()),
	})
}

pub(super) fn url_relative_to<'a>(from: UrlCow<'_>, to: UrlCow<'a>) -> Result<UrlCow<'a>> {
	use yazi_shared::url::Component::*;

	if from.is_absolute() != to.is_absolute() {
		return if to.is_absolute() {
			Ok(to)
		} else {
			bail!("Urls must be both absolute or both relative: {from:?} and {to:?}");
		};
	}

	if from.covariant(&to) {
		return Ok(UrlBuf { loc: LocBuf::zeroed("."), scheme: to.scheme().into() }.into());
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

	let buf: PathBuf = dots.chain(rest).collect();
	Ok(UrlBuf { loc: LocBuf::zeroed(buf), scheme: to.scheme().into() }.into())
}
