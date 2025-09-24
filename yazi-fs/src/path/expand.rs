use std::{borrow::Cow, ffi::{OsStr, OsString}, path::{Path, PathBuf}};

use yazi_shared::{loc::LocBuf, url::{Url, UrlBuf, UrlCow}};

use crate::{CWD, path::clean_url};

#[inline]
pub fn expand_url<'a>(url: impl Into<UrlCow<'a>>) -> UrlBuf {
	clean_url(expand_url_impl(url.into().as_url()))
}

fn expand_url_impl<'a>(url: Url<'a>) -> UrlCow<'a> {
	let (o_base, o_rest, o_urn) = url.loc.triple();

	let n_base = expand_variables(o_base);
	let n_rest = expand_variables(o_rest);
	let n_urn = expand_variables(o_urn);

	let rest_diff = n_rest.components().count() as isize - o_rest.components().count() as isize;
	let urn_diff = n_urn.components().count() as isize - o_urn.components().count() as isize;

	let uri_count = url.uri().count() as isize;
	let urn_count = url.urn().count() as isize;

	let loc = LocBuf::with(
		PathBuf::from_iter([n_base, n_rest, n_urn]),
		(uri_count + rest_diff + urn_diff) as usize,
		(urn_count + urn_diff) as usize,
	)
	.expect("Failed to create Loc from expanded path");

	let url = UrlBuf { loc, scheme: url.scheme.into() };
	match absolute_url(url.as_url()) {
		UrlCow::Borrowed { .. } => url.into(),
		c @ UrlCow::Owned { .. } => c.into_owned().into(),
	}
}

fn expand_variables(p: &Path) -> Cow<'_, Path> {
	// ${HOME} or $HOME
	#[cfg(unix)]
	let re = regex::bytes::Regex::new(r"\$(?:\{([^}]+)\}|([a-zA-Z\d_]+))").unwrap();

	// %USERPROFILE%
	#[cfg(windows)]
	let re = regex::bytes::Regex::new(r"%([^%]+)%").unwrap();

	let b = p.as_os_str().as_encoded_bytes();
	let b = re.replace_all(b, |caps: &regex::bytes::Captures| {
		let name = caps.get(2).or_else(|| caps.get(1)).unwrap();
		str::from_utf8(name.as_bytes())
			.ok()
			.and_then(std::env::var_os)
			.map_or_else(|| caps.get(0).unwrap().as_bytes().to_owned(), |s| s.into_encoded_bytes())
	});

	unsafe {
		match b {
			Cow::Borrowed(b) => Path::new(OsStr::from_encoded_bytes_unchecked(b)).into(),
			Cow::Owned(b) => PathBuf::from(OsString::from_encoded_bytes_unchecked(b)).into(),
		}
	}
}

pub fn absolute_url<'a>(url: Url<'a>) -> UrlCow<'a> {
	if url.scheme.is_virtual() {
		return url.into();
	}

	let b = url.loc.as_os_str().as_encoded_bytes();
	if cfg!(windows) && b.len() == 2 && b[1] == b':' && b[0].is_ascii_alphabetic() {
		let loc = LocBuf::with(
			format!(r"{}:\", b[0].to_ascii_uppercase() as char).into(),
			if url.has_base() { 0 } else { 2 },
			if url.has_trail() { 0 } else { 2 },
		)
		.expect("Failed to create Loc from drive letter");
		UrlBuf { loc, scheme: url.scheme.into() }.into()
	} else if let Ok(rest) = url.loc.strip_prefix("~/")
		&& let Some(home) = dirs::home_dir()
		&& home.is_absolute()
	{
		let add = home.components().count() - 1; // Home root ("~") has offset by the absolute root ("/")
		let loc = LocBuf::with(
			home.join(rest),
			url.uri().count() + if url.has_base() { 0 } else { add },
			url.urn().count() + if url.has_trail() { 0 } else { add },
		)
		.expect("Failed to create Loc from home directory");
		UrlBuf { loc, scheme: url.scheme.into() }.into()
	} else if !url.is_absolute() {
		let cwd = CWD.path();
		let loc = LocBuf::with(cwd.join(url.loc), url.uri().count(), url.urn().count())
			.expect("Failed to create Loc from relative path");
		UrlBuf { loc, scheme: url.scheme.into() }.into()
	} else {
		url.into()
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
