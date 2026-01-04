use std::path::PathBuf;

use yazi_shared::{loc::LocBuf, pool::InternStr, url::{AsUrl, Url, UrlBuf, UrlCow, UrlLike}};

use crate::CWD;

pub fn try_absolute<'a, U>(url: U) -> Option<UrlCow<'a>>
where
	U: Into<UrlCow<'a>>,
{
	try_absolute_impl(url.into())
}

fn try_absolute_impl<'a>(url: UrlCow<'a>) -> Option<UrlCow<'a>> {
	if url.kind().is_virtual() {
		return None;
	}

	let path = url.loc().as_os().expect("must be a local path");
	let b = path.as_os_str().as_encoded_bytes();

	let loc = if cfg!(windows) && b.len() == 2 && b[1] == b':' && b[0].is_ascii_alphabetic() {
		LocBuf::<PathBuf>::with(
			format!(r"{}:\", b[0].to_ascii_uppercase() as char).into(),
			if url.has_base() { 0 } else { 2 },
			if url.has_trail() { 0 } else { 2 },
		)
		.expect("Loc from drive letter")
	} else if let Ok(rest) = path.strip_prefix("~/")
		&& let Some(home) = dirs::home_dir()
		&& home.is_absolute()
	{
		let add = home.components().count() - 1; // Home root ("~") has offset by the absolute root ("/")
		LocBuf::<PathBuf>::with(
			home.join(rest),
			url.uri().components().count() + if url.has_base() { 0 } else { add },
			url.urn().components().count() + if url.has_trail() { 0 } else { add },
		)
		.expect("Loc from home directory")
	} else if !url.is_absolute() {
		LocBuf::<PathBuf>::with(
			CWD.path().join(path),
			url.uri().components().count(),
			url.urn().components().count(),
		)
		.expect("Loc from relative path")
	} else {
		return Some(url);
	};

	Some(match url.as_url() {
		Url::Regular(_) => UrlBuf::Regular(loc).into(),
		Url::Search { domain, .. } => UrlBuf::Search { loc, domain: domain.intern() }.into(),
		Url::Archive { .. } | Url::Sftp { .. } => None?,
	})
}
