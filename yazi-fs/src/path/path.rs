use std::{borrow::Cow, ffi::OsStr};

use yazi_shared::url::{UrlBuf, UrlLike};

pub fn skip_url(url: &UrlBuf, n: usize) -> Cow<'_, OsStr> {
	let mut it = url.components();
	for _ in 0..n {
		if it.next().is_none() {
			return OsStr::new("").into();
		}
	}
	it.os_str()
}

#[cfg(windows)]
pub fn backslash_to_slash(p: &std::path::Path) -> Cow<'_, std::path::Path> {
	use std::{ffi::OsString, path::PathBuf};
	let bytes = p.as_os_str().as_encoded_bytes();

	// Fast path to skip if there are no backslashes
	let skip_len = bytes.iter().take_while(|&&b| b != b'\\').count();
	if skip_len >= bytes.len() {
		return Cow::Borrowed(p);
	}

	let (skip, rest) = bytes.split_at(skip_len);
	let mut out = Vec::new();
	out.try_reserve_exact(bytes.len()).unwrap_or_else(|_| panic!());
	out.extend(skip);

	for &b in rest {
		out.push(if b == b'\\' { b'/' } else { b });
	}
	Cow::Owned(PathBuf::from(unsafe { OsString::from_encoded_bytes_unchecked(out) }))
}

#[cfg(test)]
mod tests {
	use yazi_shared::url::{AsUrl, UrlCow};

	use crate::path::url_relative_to;

	#[test]
	fn test_url_relative_to() {
		yazi_shared::init_tests();

		#[cfg(unix)]
		let cases = [
			// Same urls
			("", "", "."),
			(".", ".", "."),
			("/a", "/a", "."),
			("regular:///", "/", "."),
			("regular://", "regular://", "."),
			("regular://", "search://kw/", "search://kw/."),
			("regular:///b", "search://kw//b", "search://kw/."),
			// Relative urls
			("foo", "bar", "../bar"),
			// Absolute urls
			("/a/b/c", "/a/b", ".."),
			("/a/b", "/a/b/c", "c"),
			("/a/b/d", "/a/b/c", "../c"),
			("/a/b/c", "/a", "../.."),
			("/a/b/b", "/a/a/b", "../../a/b"),
			("regular:///a/b", "regular:///a/b/c", "c"),
			("/a/b/c/", "search://kw//a/d/", "search://kw/../../d"),
			("search://kw//a/b/c", "search://kw//a/b", "search://kw/.."),
			// Different schemes
			("", "sftp://test/", "sftp://test/"),
			("a", "sftp://test/", "sftp://test/"),
			("a", "sftp://test/b", "sftp://test/b"),
			("/a", "sftp://test//b", "sftp://test//b"),
			("sftp://test//a/b", "sftp://test//a/d", "sftp://test:0:0/../d"),
		];

		#[cfg(windows)]
		let cases = [
			(r"C:\a\b\c", r"C:\a\b", r".."),
			(r"C:\a\b", r"C:\a\b\c", "c"),
			(r"C:\a\b\d", r"C:\a\b\c", r"..\c"),
			(r"C:\a\b\c", r"C:\a", r"..\.."),
			(r"C:\a\b\b", r"C:\a\a\b", r"..\..\a\b"),
		];

		for (from, to, expected) in cases {
			let from: UrlCow = from.try_into().unwrap();
			let to: UrlCow = to.try_into().unwrap();
			assert_eq!(format!("{:?}", url_relative_to(from, to).unwrap().as_url()), expected);
		}
	}
}
