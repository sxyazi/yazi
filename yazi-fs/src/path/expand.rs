use std::borrow::Cow;

use yazi_shared::{loc::LocBuf, path::{PathBufDyn, PathCow, PathKind, PathLike}, url::{AsUrl, Url, UrlBuf, UrlCow, UrlLike}};
use yazi_shim::wtf8::FromWtf8Vec;

#[inline]
pub fn expand_url<'a>(url: impl Into<UrlCow<'a>>) -> UrlCow<'a> { expand_url_impl(url.into()) }

fn expand_url_impl(url: UrlCow) -> UrlCow {
	let (o_base, o_rest, o_urn) = url.triple();

	let n_base = expand_variables(o_base.into());
	let n_rest = expand_variables(o_rest.into());
	let n_urn = expand_variables(o_urn.into());
	if n_base.is_borrowed() && n_rest.is_borrowed() && n_urn.is_borrowed() {
		return url;
	}

	let rest_diff = n_rest.components().count() as isize - o_rest.components().count() as isize;
	let urn_diff = n_urn.components().count() as isize - o_urn.components().count() as isize;

	let uri_count = url.uri().components().count() as isize;
	let urn_count = url.urn().components().count() as isize;

	let mut path = PathBufDyn::with_capacity(url.kind(), n_base.len() + n_rest.len() + n_urn.len());
	path.try_extend([n_base, n_rest, n_urn]).expect("extend original parts should not fail");

	let uri = (uri_count + rest_diff + urn_diff) as usize;
	let urn = (urn_count + urn_diff) as usize;

	match url.as_url() {
		Url::Regular(_) => UrlBuf::Regular(
			LocBuf::<std::path::PathBuf>::with(path.into_os().unwrap(), uri, urn).unwrap(),
		),
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
		}

		let cases = [
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
}
