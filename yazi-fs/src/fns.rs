use std::{ffi::OsString, fs, io, path::{Path, PathBuf}};

use yazi_shared::url::{Component, UrlBuf, UrlLike};
use yazi_shim::OptionExt;

#[inline]
pub fn ok_or_not_found<T: Default>(result: io::Result<T>) -> io::Result<T> {
	match result {
		Ok(t) => Ok(t),
		Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(T::default()),
		Err(_) => result,
	}
}

pub fn unique_dir(mut path: PathBuf) -> io::Result<PathBuf> {
	let mut err = match fs::create_dir(&path) {
		Ok(()) => return Ok(path),
		Err(e) if e.kind() == io::ErrorKind::AlreadyExists => e,
		Err(e) => return Err(e),
	};

	let Some(name) = path.file_name().owned() else {
		return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty directory name"));
	};

	let mut buf = OsString::with_capacity(name.len() + 5);
	for i in 1..=1000 {
		buf.clear();
		buf.push(&name);
		buf.push(format!("_{i}"));

		path.set_file_name(&buf);
		match fs::create_dir(&path) {
			Ok(()) => return Ok(path),
			Err(e) if e.kind() == io::ErrorKind::AlreadyExists => err = e,
			Err(e) => return Err(e),
		}
	}

	Err(err)
}

pub fn unique_file(
	mut path: PathBuf,
	options: &fs::OpenOptions,
) -> io::Result<(PathBuf, fs::File)> {
	let mut options = options.clone();
	let mut err = match options.create_new(true).open(&path) {
		Ok(file) => return Ok((path, file)),
		Err(e) if e.kind() == io::ErrorKind::AlreadyExists => e,
		Err(e) => return Err(e),
	};

	let Some(stem) = path.file_stem().owned() else {
		return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty file stem"));
	};

	let ext = path.extension().owned();
	let mut buf = OsString::with_capacity(stem.len() + ext.as_ref().map_or(0, |s| s.len()) + 6);

	for i in 1..=1000 {
		buf.clear();
		buf.push(&stem);
		buf.push(format!("_{i}"));

		if let Some(e) = &ext {
			buf.push(".");
			buf.push(e);
		}

		path.set_file_name(&buf);
		match options.open(&path) {
			Ok(file) => return Ok((path, file)),
			Err(e) if e.kind() == io::ErrorKind::AlreadyExists => err = e,
			Err(e) => return Err(e),
		}
	}

	Err(err)
}

// Find the max common root in a list of urls
// e.g. /a/b/c, /a/b/d       -> /a/b
//      /aa/bb/cc, /aa/dd/ee -> /aa
pub fn max_common_root(urls: &[UrlBuf]) -> usize {
	if urls.is_empty() {
		return 0;
	} else if urls.len() == 1 {
		return urls[0].components().count() - 1;
	}

	let mut it = urls.iter().map(|u| u.parent());
	let Some(first) = it.next().unwrap() else {
		return 0; // The first URL has no parent
	};

	let mut common = first.components().count();
	for parent in it {
		let Some(parent) = parent else {
			return 0; // One of the URLs has no parent
		};

		common = first
			.components()
			.zip(parent.components())
			.take_while(|(a, b)| match (a, b) {
				(Component::Auth(a), Component::Auth(b)) => a.covariant(b),
				(a, b) => a == b,
			})
			.count()
			.min(common);

		if common == 0 {
			break; // No common root found
		}
	}

	common
}

#[cfg(unix)]
#[test]
fn test_max_common_root() {
	yazi_shared::init_tests();

	fn assert(input: &[&str], expected: &str) {
		use std::{ffi::OsStr, str::FromStr};
		let urls: Vec<_> =
			input.iter().copied().map(UrlBuf::from_str).collect::<Result<_, _>>().unwrap();

		let mut comp = urls[0].components();
		for _ in 0..comp.clone().count() - max_common_root(&urls) {
			comp.next_back();
		}
		assert_eq!(comp.os_str(), OsStr::new(expected));
	}

	assert_eq!(max_common_root(&[]), 0);
	assert(&[""], "");
	assert(&["a"], "");
	assert(
		&["test-view://fx:1:1/@Ds2kw0A//a", "test-view://fx:1:1/@Ds2kw0A//b"],
		"test-view://fx/@Ds2kw0A//",
	);
	assert(&["test-hub://a1/@root/a", "test-hub://b1/@root/b"], "test-hub://root/@/");

	assert(&["/a"], "/");
	assert(&["/a/b"], "/a");
	assert(&["/a/b/c", "/a/b/d"], "/a/b");
	assert(&["/aa/bb/cc", "/aa/dd/ee"], "/aa");
	assert(&["/aa/bb/cc", "/aa/bb/cc/dd/ee", "/aa/bb/cc/ff"], "/aa/bb");
}

pub async fn create_owned_dir(p: &Path) -> io::Result<()> {
	let p = p.to_owned();
	tokio::task::spawn_blocking(move || create_owned_dir_blocking(&p)).await?
}

pub fn create_owned_dir_blocking(p: &Path) -> io::Result<()> {
	#[cfg(unix)]
	{
		use std::{fs::{DirBuilder, OpenOptions}, os::unix::fs::{DirBuilderExt, OpenOptionsExt}};

		use rustix::fs::{self, Mode, OFlags};

		DirBuilder::new().mode(0o700).recursive(true).create(p)?;

		// Open and stat the directory.
		let dir = OpenOptions::new()
			.read(true)
			.custom_flags((OFlags::DIRECTORY | OFlags::NOFOLLOW).bits() as _)
			.open(p)?;
		let stat = fs::fstat(&dir)?;

		// Reject directories not owned by the current user.
		let uid = yazi_shim::Uzers::uid();
		if stat.st_uid != uid {
			return Err(io::Error::new(
				io::ErrorKind::PermissionDenied,
				format!("directory {p:?} is owned by uid {} but current uid is {uid}", stat.st_uid),
			));
		}

		// Enforce mode 0o700 via the fd.
		Ok(fs::fchmod(&dir, Mode::RWXU)?)
	}
	#[cfg(not(unix))]
	{
		std::fs::DirBuilder::new().recursive(true).create(p)
	}
}
