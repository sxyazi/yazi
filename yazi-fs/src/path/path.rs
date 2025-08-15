use std::{borrow::Cow, ffi::{OsStr, OsString}, future::Future, io, path::PathBuf};

use anyhow::{Result, bail};
use yazi_shared::{loc::LocBuf, url::UrlBuf};

use crate::provider;

pub fn skip_url(url: &UrlBuf, n: usize) -> Cow<'_, OsStr> {
	let mut it = url.components();
	for _ in 0..n {
		if it.next().is_none() {
			return OsStr::new("").into();
		}
	}
	it.os_str()
}

pub async fn unique_name<F>(u: UrlBuf, append: F) -> io::Result<UrlBuf>
where
	F: Future<Output = bool>,
{
	match provider::symlink_metadata(&u).await {
		Ok(_) => _unique_name(u, append.await).await,
		Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(u),
		Err(e) => Err(e),
	}
}

async fn _unique_name(mut url: UrlBuf, append: bool) -> io::Result<UrlBuf> {
	let Some(stem) = url.file_stem().map(|s| s.to_owned()) else {
		return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty file stem"));
	};

	let dot_ext = url.extension().map_or_else(OsString::new, |e| {
		let mut s = OsString::with_capacity(e.len() + 1);
		s.push(".");
		s.push(e);
		s
	});

	let mut i = 1u64;
	let mut name = OsString::with_capacity(stem.len() + dot_ext.len() + 5);
	loop {
		name.clear();
		name.push(&stem);

		if append {
			name.push(&dot_ext);
			name.push(format!("_{i}"));
		} else {
			name.push(format!("_{i}"));
			name.push(&dot_ext);
		}

		url.set_name(&name);
		match provider::symlink_metadata(&url).await {
			Ok(_) => i += 1,
			Err(e) if e.kind() == io::ErrorKind::NotFound => break,
			Err(e) => return Err(e),
		}
	}

	Ok(url)
}

pub fn url_relative_to<'a>(from: &UrlBuf, to: &'a UrlBuf) -> Result<Cow<'a, UrlBuf>> {
	use yazi_shared::url::Component::*;

	if from.is_absolute() != to.is_absolute() {
		return if to.is_absolute() {
			Ok(to.into())
		} else {
			bail!("Urls must be both absolute or both relative: {from:?} and {to:?}");
		};
	}

	if from.covariant(to) {
		return Ok(UrlBuf { loc: LocBuf::zeroed("."), scheme: to.scheme.clone() }.into());
	}

	let (mut f_it, mut t_it) = (from.components(), to.components());
	let (f_head, t_head) = loop {
		match (f_it.next(), t_it.next()) {
			(Some(Scheme(a)), Some(Scheme(b))) if a.covariant(b) => {}
			(Some(RootDir), Some(RootDir)) => {}
			(Some(Prefix(a)), Some(Prefix(b))) if a == b => {}
			(Some(Scheme(_) | Prefix(_) | RootDir), _) | (_, Some(Scheme(_) | Prefix(_) | RootDir)) => {
				return Ok(to.into());
			}
			(None, None) => break (None, None),
			(a, b) if a != b => break (a, b),
			_ => (),
		}
	};

	let dots = f_head.into_iter().chain(f_it).map(|_| ParentDir);
	let rest = t_head.into_iter().chain(t_it);

	let buf: PathBuf = dots.chain(rest).collect();
	Ok(UrlBuf { loc: LocBuf::zeroed(buf), scheme: to.scheme.clone() }.into())
}

#[cfg(windows)]
pub fn backslash_to_slash(p: &std::path::Path) -> Cow<'_, std::path::Path> {
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
	use std::borrow::Cow;

	use super::url_relative_to;

	#[test]
	fn test_path_relative_to() {
		fn assert(from: &str, to: &str, ret: &str) {
			assert_eq!(
				url_relative_to(&from.parse().unwrap(), &to.parse().unwrap()).unwrap(),
				Cow::Owned(ret.parse().unwrap())
			);
		}

		#[cfg(unix)]
		{
			// Same urls
			assert("", "", ".");
			assert(".", ".", ".");
			assert("/a", "/a", ".");
			assert("regular:///", "/", ".");
			assert("regular://", "regular://", ".");
			assert("regular://", "search://kw/", "search://kw/.");
			assert("regular:///b", "search://kw//b", "search://kw/.");

			// Relative urls
			assert("foo", "bar", "../bar");

			// Absolute urls
			assert("/a/b/c", "/a/b", "../");
			assert("/a/b", "/a/b/c", "c");
			assert("/a/b/d", "/a/b/c", "../c");
			assert("/a/b/c", "/a", "../../");
			assert("/a/b/b", "/a/a/b", "../../a/b");

			assert("regular:///a/b", "regular:///a/b/c", "c");
			assert("/a/b/c/", "search://kw//a/d/", "search://kw/../../d");
			assert("search://kw//a/b/c", "search://kw//a/b", "search://kw/../");

			// Different schemes
			assert("", "sftp://test/", "sftp://test/");
			assert("a", "sftp://test/", "sftp://test/");
			assert("a", "sftp://test/b", "sftp://test/b");
			assert("/a", "sftp://test//b", "sftp://test//b");
			assert("sftp://test//a/b", "sftp://test//a/d", "sftp://test/../d");
		}
		#[cfg(windows)]
		{
			assert(r"C:\a\b\c", r"C:\a\b", r"..\");
			assert(r"C:\a\b", r"C:\a\b\c", "c");
			assert(r"C:\a\b\d", r"C:\a\b\c", r"..\c");
			assert(r"C:\a\b\c", r"C:\a", r"..\..\");
			assert(r"C:\a\b\b", r"C:\a\a\b", r"..\..\a\b");
		}
	}
}
