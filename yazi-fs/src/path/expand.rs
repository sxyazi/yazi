use std::borrow::Cow;

use yazi_shared::{loc::LocBuf, path::{Component, PathBufDyn, PathCow, PathKind, PathLike}, url::{AsUrl, Url, UrlBuf, UrlCow, UrlLike}};
use yazi_shim::wtf8::FromWtf8Vec;

#[inline]
pub fn expand_url<'a>(url: impl Into<UrlCow<'a>>) -> UrlCow<'a> { expand_url_impl(url.into()) }

fn expand_url_impl(url: UrlCow) -> UrlCow {
	let (base, rest, urn) = url.triple();

	let base = expand_variables(base.into());
	let rest = expand_variables(rest.into());
	let urn = expand_variables(urn.into());
	if base.is_borrowed() && rest.is_borrowed() && urn.is_borrowed() {
		return url;
	}

	let mut path = PathBufDyn::with_capacity(url.kind(), base.len() + rest.len() + urn.len());
	path.try_push(&base).expect("push original base should not fail");
	let c_base = path.components().count();

	path.try_push(&rest).expect("push original URI should not fail");
	let c_trail = path.components().count();

	path.try_push(&urn).expect("push original URN should not fail");
	let c_full = path.components().count();

	let uri = if urn.has_prefix() || rest.has_prefix() {
		c_full
	} else if urn.has_root() || rest.has_root() {
		c_full - c_base.min(path.has_prefix() as usize)
	} else {
		c_full - c_base
	};
	let urn = if urn.has_prefix() || urn.has_root() {
		path.components().rev().take_while(|&c| c != Component::RootDir).count()
	} else {
		c_full - c_trail
	};

	match url.as_url() {
		Url::Regular(_) => UrlBuf::from(path.into_os().unwrap()),
		Url::Search { auth, .. } => UrlBuf::Search {
			loc:  LocBuf::<std::path::PathBuf>::with(path.into_os().unwrap(), uri, urn).unwrap(),
			auth: auth.clone(),
		},
		Url::Mount { auth, .. } => UrlBuf::Mount {
			loc:  LocBuf::<std::path::PathBuf>::with(path.into_os().unwrap(), uri, urn).unwrap(),
			auth: auth.clone(),
		},
		Url::Hub { auth, .. } => UrlBuf::Hub {
			auth: auth.clone().with_parent_depth(path.components().auth_depth()),
			loc:  LocBuf::<std::path::PathBuf>::with(path.into_os().unwrap(), uri, urn).unwrap(),
		},
		Url::Scope { auth, .. } => UrlBuf::Scope {
			loc:  LocBuf::<typed_path::UnixPathBuf>::with(path.into_unix().unwrap(), uri, urn).unwrap(),
			auth: auth.clone(),
		},
		Url::Sftp { auth, .. } => UrlBuf::Sftp {
			loc:  LocBuf::<typed_path::UnixPathBuf>::with(path.into_unix().unwrap(), uri, urn).unwrap(),
			auth: auth.clone(),
		},
	}
	.into()
}

fn expand_variables(p: PathCow) -> PathCow {
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

	match (b, p.kind()) {
		(Cow::Borrowed(_), _) => p,
		(Cow::Owned(b), PathKind::Os) => {
			PathBufDyn::Os(std::path::PathBuf::from_wtf8_vec(b).expect("valid WTF-8 path")).into()
		}
		(Cow::Owned(b), PathKind::Unix) => PathBufDyn::Unix(b.into()).into(),
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

			std::env::set_var("ABS", "/tmp/downloads");
			std::env::set_var("DOT", ".");
			std::env::set_var("DOT_FOO", "./foo");
		}

		let cases = [
			// Absolute path expanded
			("$ABS", "/tmp/downloads"),
			("test-mount://7z:1//$ABS", "test-mount://7z:3//tmp/downloads"),
			("test-mount://7z:1:1//$ABS", "test-mount://7z:3:2//tmp/downloads"),
			("test-scope://aws//$ABS", "test-scope://aws:3:2//tmp/downloads"),
			// Current directory expanded
			("test-mount://7z:1/$DOT", "test-mount://7z:1/."),
			("test-mount://7z:1//$DOT", "test-mount://7z//"),
			("test-mount://7z:1:1/$DOT", "test-mount://7z:1:1/."),
			("test-mount://7z:1:1//$DOT", "test-mount://7z//"),
			("test-mount://7z:2:1//base/$DOT/file", "test-mount://7z:1:1//base/./file"),
			("test-mount://7z:1:1/$DOT_FOO", "test-mount://7z:2:2/./foo"),
			("test-mount://7z:1:1//$DOT_FOO", "test-mount://7z:1:1//./foo"),
			("test-scope://aws/$DOT", "test-scope://aws:1:1/."),
			("test-scope://aws//$DOT", "test-scope://aws//."),
			// Zero extra component expanded
			("test-mount://7z//tmp/test.zip/$FOO/bar", "test-mount://7z//tmp/test.zip/foo/bar"),
			("test-mount://7z:1//tmp/test.zip/$FOO/bar", "test-mount://7z:1//tmp/test.zip/foo/bar"),
			("test-mount://7z:2//tmp/test.zip/bar/$FOO", "test-mount://7z:2//tmp/test.zip/bar/foo"),
			("test-mount://7z:3//tmp/test.zip/$FOO/bar", "test-mount://7z:3//tmp/test.zip/foo/bar"),
			("test-mount://7z:3:1//tmp/test.zip/bar/$FOO", "test-mount://7z:3:1//tmp/test.zip/bar/foo"),
			("test-mount://7z:3:2//tmp/test.zip/$FOO/bar", "test-mount://7z:3:2//tmp/test.zip/foo/bar"),
			("test-mount://7z:3:3//tmp/test.zip/bar/$FOO", "test-mount://7z:3:3//tmp/test.zip/bar/foo"),
			// +1 component
			("test-mount://7z//tmp/test.zip/$BAR_BAZ", "test-mount://7z//tmp/test.zip/bar/baz"),
			("test-mount://7z:1//tmp/test.zip/$BAR_BAZ", "test-mount://7z:2//tmp/test.zip/bar/baz"),
			("test-mount://7z:2//$BAR_BAZ/tmp/test.zip", "test-mount://7z:2//bar/baz/tmp/test.zip"),
			("test-mount://7z:2:1//tmp/test.zip/$BAR_BAZ", "test-mount://7z:3:2//tmp/test.zip/bar/baz"),
			("test-mount://7z:2:2//tmp/$BAR_BAZ/test.zip", "test-mount://7z:3:3//tmp/bar/baz/test.zip"),
			("test-mount://7z:2:2//$BAR_BAZ/tmp/test.zip", "test-mount://7z:2:2//bar/baz/tmp/test.zip"),
			("test-hub://a1/@root/$BAR_BAZ", "test-hub://a1:2:2/@,root/bar/baz"),
			// -1 component
			("test-mount://7z//tmp/test.zip/${BAR/BAZ}", "test-mount://7z//tmp/test.zip/bar_baz"),
			("test-mount://7z:1//tmp/test.zip/${BAR/BAZ}", "test-mount://7z:1//tmp/test.zip/${BAR/BAZ}"),
			("test-mount://7z:1//tmp/${BAR/BAZ}/test.zip", "test-mount://7z:1//tmp/bar_baz/test.zip"),
			("test-mount://7z:2//tmp/test.zip/${BAR/BAZ}", "test-mount://7z:1//tmp/test.zip/bar_baz"),
			("test-mount://7z:2//tmp/${BAR/BAZ}/test.zip", "test-mount://7z:2//tmp/${BAR/BAZ}/test.zip"),
			(
				"test-mount://7z:2:1//tmp/test.zip/${BAR/BAZ}",
				"test-mount://7z:2:1//tmp/test.zip/${BAR/BAZ}",
			),
			(
				"test-mount://7z:2:1//tmp/${BAR/BAZ}/test.zip",
				"test-mount://7z:2:1//tmp/${BAR/BAZ}/test.zip",
			),
			("test-mount://7z:2:1//${BAR/BAZ}/tmp/test.zip", "test-mount://7z:2:1//bar_baz/tmp/test.zip"),
			("test-mount://7z:3:2//tmp/test.zip/${BAR/BAZ}", "test-mount://7z:2:1//tmp/test.zip/bar_baz"),
			(
				"test-mount://7z:3:2//tmp/${BAR/BAZ}/test.zip",
				"test-mount://7z:3:2//tmp/${BAR/BAZ}/test.zip",
			),
			("test-mount://7z:3:3//tmp/test.zip/${BAR/BAZ}", "test-mount://7z:2:2//tmp/test.zip/bar_baz"),
			("test-mount://7z:3:3//tmp/${BAR/BAZ}/test.zip", "test-mount://7z:2:2//tmp/bar_baz/test.zip"),
			("test-hub://a1:2:2/@b1,root/${BAR/BAZ}", "test-hub://a1/@root/bar_baz"),
			// Zeros all components
			("test-mount://7z//${EM/PT/Y}", "test-mount://7z//"),
			("test-mount://7z:1//${EM/PT/Y}", "test-mount://7z:1//${EM/PT/Y}"),
			("test-mount://7z:2//${EM/PT/Y}", "test-mount://7z:2//${EM/PT/Y}"),
			("test-mount://7z:3//${EM/PT/Y}", "test-mount://7z//"),
			("test-mount://7z:4//${EM/PT/Y}", "test-mount://7z:1//"),
		];

		for (input, expected) in cases {
			let u: UrlBuf = input.parse()?;
			assert_eq!(format!("{:?}", expand_url(u).as_url()), expected);
		}

		Ok(())
	}

	#[cfg(windows)]
	#[test]
	fn test_expand_url() -> Result<()> {
		yazi_shared::init_tests();
		unsafe {
			std::env::set_var("ROOTED", r"\downloads");
			std::env::set_var("DRIVE_ABS", r"C:\downloads");
			std::env::set_var("DRIVE_REL", r"C:downloads");
		}

		let cases = [
			// Rooted path expanded
			(r"test-mount://7z:2:1/D:\base\%ROOTED%\file", r"test-mount://7z:3:1/D:\downloads\file"),
			(r"test-mount://7z:2:1/D:\base\file\%ROOTED%", r"test-mount://7z:2:1/D:\downloads"),
			// Drive-absolute path expanded
			(r"test-mount://7z:2:1/D:\base\%DRIVE_ABS%\file", r"test-mount://7z:4:1/C:\downloads\file"),
			(r"test-mount://7z:2:1/D:\base\file\%DRIVE_ABS%", r"test-mount://7z:3:1/C:\downloads"),
			// Drive-relative path expanded
			(r"test-mount://7z:2:1/D:\base\%DRIVE_REL%\file", r"test-mount://7z:3:1/C:downloads\file"),
			(r"test-mount://7z:2:1/D:\base\file\%DRIVE_REL%", r"test-mount://7z:2:2/C:downloads"),
		];

		for (input, expected) in cases {
			let u: UrlBuf = input.parse()?;
			assert_eq!(format!("{:?}", expand_url(u).as_url()), expected);
		}

		Ok(())
	}
}
