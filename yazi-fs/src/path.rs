use std::{borrow::Cow, env, ffi::{OsStr, OsString}, future::Future, io, path::{Path, PathBuf}};

use anyhow::{Result, bail};
use yazi_shared::url::{Loc, Url};

use crate::{CWD, services};

pub fn clean_url<'a>(url: impl Into<Cow<'a, Url>>) -> Cow<'a, Url> {
	let url = url.into();
	let path = clean_path(&url.loc);

	if path.as_os_str() == url.loc.as_os_str() {
		url
	} else {
		url.with(Loc::with(&clean_path(url.loc.base()), path)).into()
	}
}

fn clean_path(path: &Path) -> PathBuf {
	use std::path::Component::*;

	let mut out = vec![];
	for c in path.components() {
		match c {
			CurDir => {}
			ParentDir => match out.last() {
				Some(RootDir) => {}
				Some(Normal(_)) => _ = out.pop(),
				None | Some(CurDir) | Some(ParentDir) | Some(Prefix(_)) => out.push(c),
			},
			c => out.push(c),
		}
	}

	if out.is_empty() { PathBuf::from(".") } else { out.iter().collect() }
}

// FIXME: VFS
#[inline]
pub fn expand_path(p: impl AsRef<Path>) -> PathBuf {
	expand_url(Url::from(p.as_ref())).into_owned().loc.into_path()
}

#[inline]
pub fn expand_url<'a>(url: impl Into<Cow<'a, Url>>) -> Cow<'a, Url> {
	let cow: Cow<'a, Url> = url.into();
	match _expand_url(&cow) {
		Cow::Borrowed(_) => cow,
		Cow::Owned(url) => url.into(),
	}
}

fn _expand_url(url: &Url) -> Cow<'_, Url> {
	// ${HOME} or $HOME
	#[cfg(unix)]
	let re = regex::bytes::Regex::new(r"\$(?:\{([^}]+)\}|([a-zA-Z\d_]+))").unwrap();

	// %USERPROFILE%
	#[cfg(windows)]
	let re = regex::bytes::Regex::new(r"%([^%]+)%").unwrap();

	let b = url.loc.as_os_str().as_encoded_bytes();
	let local = !url.scheme.is_virtual();

	// Windows paths that only have a drive letter but no root, e.g. "D:"
	#[cfg(windows)]
	if local && b.len() == 2 && b[1] == b':' && b[0].is_ascii_alphabetic() {
		url.with(format!(r"{}:\", b[0].to_ascii_uppercase() as char)).into();
	}

	let b = re.replace_all(b, |caps: &regex::bytes::Captures| {
		let name = caps.get(2).or_else(|| caps.get(1)).unwrap();
		str::from_utf8(name.as_bytes())
			.ok()
			.and_then(env::var_os)
			.map_or_else(|| caps.get(0).unwrap().as_bytes().to_owned(), |s| s.into_encoded_bytes())
	});
	if matches!(b, Cow::Borrowed(_)) {
		return url.into();
	}

	let p = unsafe { PathBuf::from(OsString::from_encoded_bytes_unchecked(b.into_owned())) };
	if let Some(rest) = p.strip_prefix("~").ok().filter(|_| local) {
		url.with(clean_path(&dirs::home_dir().unwrap_or_default().join(rest))).into()
	} else if p.is_absolute() {
		url.with(clean_path(&p)).into()
	} else {
		clean_url(CWD.load().join(p))
	}
}

pub fn skip_url(url: &Url, n: usize) -> Cow<'_, OsStr> {
	let mut it = url.components();
	for _ in 0..n {
		if it.next().is_none() {
			return OsStr::new("").into();
		}
	}
	it.os_str()
}

pub async fn unique_name<F>(u: Url, append: F) -> io::Result<Url>
where
	F: Future<Output = bool>,
{
	match services::symlink_metadata(&u).await {
		Ok(_) => _unique_name(u, append.await).await,
		Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(u),
		Err(e) => Err(e),
	}
}

async fn _unique_name(mut url: Url, append: bool) -> io::Result<Url> {
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
			name.push("_");
			name.push(i.to_string());
		} else {
			name.push("_");
			name.push(i.to_string());
			name.push(&dot_ext);
		}

		url.set_name(&name);
		match services::symlink_metadata(&url).await {
			Ok(_) => i += 1,
			Err(e) if e.kind() == io::ErrorKind::NotFound => break,
			Err(e) => return Err(e),
		}
	}

	Ok(url)
}

pub fn url_relative_to<'a>(from: &Url, to: &'a Url) -> Result<Cow<'a, Url>> {
	use yazi_shared::url::Component::*;

	if from.is_absolute() != to.is_absolute() {
		return if to.is_absolute() {
			Ok(to.into())
		} else {
			bail!("Urls must be both absolute or both relative: {from:?} and {to:?}");
		};
	}

	if from.covariant(to) {
		return Ok(to.with(Path::new(".")).into());
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
	Ok(to.with(buf).into())
}

#[cfg(windows)]
pub fn backslash_to_slash(p: &Path) -> Cow<'_, Path> {
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

	use yazi_shared::url::Url;

	use super::url_relative_to;

	#[test]
	fn test_path_relative_to() {
		fn assert(from: &str, to: &str, ret: &str) {
			assert_eq!(
				url_relative_to(&Url::try_from(from).unwrap(), &Url::try_from(to).unwrap()).unwrap(),
				Cow::Owned(Url::try_from(ret).unwrap())
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
