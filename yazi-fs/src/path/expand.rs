use std::{borrow::Cow, path::PathBuf};

use yazi_shared::{FromWtf8Vec, loc::LocBuf, path::{AsPath, PathBufDyn, PathCow, PathDyn, PathLike}, pool::InternStr, url::{AsUrl, Url, UrlBuf, UrlCow, UrlLike}};

use crate::{CWD, path::clean_url};

#[inline]
pub fn expand_url<'a>(url: impl Into<UrlCow<'a>>) -> UrlBuf {
	clean_url(expand_url_impl(url.into().as_url()))
}

fn expand_url_impl<'a>(url: Url<'a>) -> UrlCow<'a> {
	let (o_base, o_rest, o_urn) = url.triple();

	let n_base = expand_variables(o_base);
	let n_rest = expand_variables(o_rest);
	let n_urn = expand_variables(o_urn);

	let rest_diff =
		n_rest.as_path().components().count() as isize - o_rest.components().count() as isize;
	let urn_diff =
		n_urn.as_path().components().count() as isize - o_urn.components().count() as isize;

	let uri_count = url.uri().components().count() as isize;
	let urn_count = url.urn().components().count() as isize;

	let mut path = PathBufDyn::with_capacity(
		url.kind(),
		n_base.as_path().len() + n_rest.as_path().len() + n_urn.as_path().len(),
	);
	path.try_extend([n_base, n_rest, n_urn]).expect("extend original parts should not fail");

	let loc = LocBuf::<PathBuf>::with(
		path.into_os().expect("Failed to convert PathBufDyn to PathBuf"),
		(uri_count + rest_diff + urn_diff) as usize,
		(urn_count + urn_diff) as usize,
	)
	.expect("Failed to create Loc from expanded path");

	let expanded = match url {
		Url::Regular(_) => UrlBuf::Regular(loc),
		Url::Search { domain, .. } => UrlBuf::Search { loc, domain: domain.intern() },
		Url::Archive { domain, .. } => UrlBuf::Archive { loc, domain: domain.intern() },
		Url::Sftp { domain, .. } => UrlBuf::Sftp { loc, domain: domain.intern() },
	};

	absolute_url(expanded)
}

fn expand_variables<'a>(p: PathDyn<'a>) -> PathCow<'a> {
	// ${HOME} or $HOME
	#[cfg(unix)]
	let re = regex::bytes::Regex::new(r"\$(?:\{([^}]+)\}|([a-zA-Z\d_]+))").unwrap();

	// %USERPROFILE%
	#[cfg(windows)]
	let re = regex::bytes::Regex::new(r"%([^%]+)%").unwrap();

	let b = p.encoded_bytes();
	let b = re.replace_all(b, |caps: &regex::bytes::Captures| {
		let name = caps.get(2).or_else(|| caps.get(1)).unwrap();
		str::from_utf8(name.as_bytes())
			.ok()
			.and_then(std::env::var_os)
			.map_or_else(|| caps.get(0).unwrap().as_bytes().to_owned(), |s| s.into_encoded_bytes())
	});

	match (b, p) {
		(Cow::Borrowed(_), _) => p.into(),
		(Cow::Owned(b), PathDyn::Os(_)) => {
			PathBufDyn::Os(std::path::PathBuf::from_wtf8_vec(b).expect("valid WTF-8 path")).into()
		}
	}
}

pub fn absolute_url<'a>(url: impl Into<UrlCow<'a>>) -> UrlCow<'a> { absolute_url_impl(url.into()) }

fn absolute_url_impl<'a>(url: UrlCow<'a>) -> UrlCow<'a> {
	if url.kind().is_virtual() {
		return url.into();
	}

	let path = url.loc().as_os().expect("must be a local path");
	let b = path.as_os_str().as_encoded_bytes();

	let loc = if cfg!(windows) && b.len() == 2 && b[1] == b':' && b[0].is_ascii_alphabetic() {
		LocBuf::<PathBuf>::with(
			format!(r"{}:\", b[0].to_ascii_uppercase() as char).into(),
			if url.has_base() { 0 } else { 2 },
			if url.has_trail() { 0 } else { 2 },
		)
		.expect("Failed to create Loc from drive letter")
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
		.expect("Failed to create Loc from home directory")
	} else if !url.is_absolute() {
		let cwd = CWD.path();
		LocBuf::<PathBuf>::with(
			cwd.join(path),
			url.uri().components().count(),
			url.urn().components().count(),
		)
		.expect("Failed to create Loc from relative path")
	} else {
		return url;
	};

	match url.as_url() {
		Url::Regular(_) => UrlBuf::Regular(loc).into(),
		Url::Search { domain, .. } => UrlBuf::Search { loc, domain: domain.intern() }.into(),
		Url::Archive { .. } | Url::Sftp { .. } => unreachable!(),
	}
}

#[cfg(test)]
mod tests {
	use anyhow::Result;

	use super::*;

	#[cfg(unix)]
	#[test]
	fn test_expand_url() -> Result<()> {
		yazi_shared::init_tests();
		unsafe {
			std::env::set_var("FOO", "foo");
			std::env::set_var("BAR_BAZ", "bar/baz");
			std::env::set_var("BAR/BAZ", "bar_baz");
			std::env::set_var("EM/PT/Y", "");
		}

		let cases = [
			// Zero extra component expanded
			("archive:////tmp/test.zip/$FOO/bar", "archive:////tmp/test.zip/foo/bar"),
			("archive://:1//tmp/test.zip/$FOO/bar", "archive://:1//tmp/test.zip/foo/bar"),
			("archive://:2//tmp/test.zip/bar/$FOO", "archive://:2//tmp/test.zip/bar/foo"),
			("archive://:3//tmp/test.zip/$FOO/bar", "archive://:3//tmp/test.zip/foo/bar"),
			("archive://:3:1//tmp/test.zip/bar/$FOO", "archive://:3:1//tmp/test.zip/bar/foo"),
			("archive://:3:2//tmp/test.zip/$FOO/bar", "archive://:3:2//tmp/test.zip/foo/bar"),
			("archive://:3:3//tmp/test.zip/bar/$FOO", "archive://:3:3//tmp/test.zip/bar/foo"),
			// +1 component
			("archive:////tmp/test.zip/$BAR_BAZ", "archive:////tmp/test.zip/bar/baz"),
			("archive://:1//tmp/test.zip/$BAR_BAZ", "archive://:2//tmp/test.zip/bar/baz"),
			("archive://:2//$BAR_BAZ/tmp/test.zip", "archive://:2//bar/baz/tmp/test.zip"),
			("archive://:2:1//tmp/test.zip/$BAR_BAZ", "archive://:3:2//tmp/test.zip/bar/baz"),
			("archive://:2:2//tmp/$BAR_BAZ/test.zip", "archive://:3:3//tmp/bar/baz/test.zip"),
			("archive://:2:2//$BAR_BAZ/tmp/test.zip", "archive://:2:2//bar/baz/tmp/test.zip"),
			// -1 component
			("archive:////tmp/test.zip/${BAR/BAZ}", "archive:////tmp/test.zip/bar_baz"),
			("archive://:1//tmp/test.zip/${BAR/BAZ}", "archive://:1//tmp/test.zip/${BAR/BAZ}"),
			("archive://:1//tmp/${BAR/BAZ}/test.zip", "archive://:1//tmp/bar_baz/test.zip"),
			("archive://:2//tmp/test.zip/${BAR/BAZ}", "archive://:1//tmp/test.zip/bar_baz"),
			("archive://:2//tmp/${BAR/BAZ}/test.zip", "archive://:2//tmp/${BAR/BAZ}/test.zip"),
			("archive://:2:1//tmp/test.zip/${BAR/BAZ}", "archive://:2:1//tmp/test.zip/${BAR/BAZ}"),
			("archive://:2:1//tmp/${BAR/BAZ}/test.zip", "archive://:2:1//tmp/${BAR/BAZ}/test.zip"),
			("archive://:2:1//${BAR/BAZ}/tmp/test.zip", "archive://:2:1//bar_baz/tmp/test.zip"),
			("archive://:3:2//tmp/test.zip/${BAR/BAZ}", "archive://:2:1//tmp/test.zip/bar_baz"),
			("archive://:3:2//tmp/${BAR/BAZ}/test.zip", "archive://:3:2//tmp/${BAR/BAZ}/test.zip"),
			("archive://:3:3//tmp/test.zip/${BAR/BAZ}", "archive://:2:2//tmp/test.zip/bar_baz"),
			("archive://:3:3//tmp/${BAR/BAZ}/test.zip", "archive://:2:2//tmp/bar_baz/test.zip"),
			// Zeros all components
			("archive:////${EM/PT/Y}", "archive:////"),
			("archive://:1//${EM/PT/Y}", "archive://:1//${EM/PT/Y}"),
			("archive://:2//${EM/PT/Y}", "archive://:2//${EM/PT/Y}"),
			("archive://:3//${EM/PT/Y}", "archive:////"),
			("archive://:4//${EM/PT/Y}", "archive://:1//"),
		];

		for (input, expected) in cases {
			let u: UrlBuf = input.parse()?;
			assert_eq!(format!("{:?}", expand_url(u)), expected);
		}

		Ok(())
	}
}
