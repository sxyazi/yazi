use std::{io, path::Path};

use yazi_shared::url::{Component, UrlBuf, UrlLike};

#[inline]
pub fn ok_or_not_found<T: Default>(result: io::Result<T>) -> io::Result<T> {
	match result {
		Ok(t) => Ok(t),
		Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(T::default()),
		Err(_) => result,
	}
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
				(Component::Scheme(a), Component::Scheme(b)) => a.covariant(*b),
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
		use std::{fs::{DirBuilder, OpenOptions}, mem, os::unix::{fs::{DirBuilderExt, OpenOptionsExt}, io::AsRawFd}};

		use libc::{O_DIRECTORY, O_NOFOLLOW};
		use uzers::Users;
		use yazi_shared::USERS_CACHE;

		DirBuilder::new().mode(0o700).recursive(true).create(p)?;
		let dir = OpenOptions::new().read(true).custom_flags(O_DIRECTORY | O_NOFOLLOW).open(p)?;

		let mut stat: libc::stat = unsafe { mem::zeroed() };
		if unsafe { libc::fstat(dir.as_raw_fd(), &mut stat) } != 0 {
			return Err(io::Error::last_os_error());
		}

		// Reject directories not owned by the current user.
		let uid = USERS_CACHE.get_current_uid();
		if stat.st_uid != uid {
			return Err(io::Error::new(
				io::ErrorKind::PermissionDenied,
				format!("directory {:?} is owned by uid {} but current uid is {}", p, stat.st_uid, uid),
			));
		}

		// Enforce mode 0o700 via the fd.
		if unsafe { libc::fchmod(dir.as_raw_fd(), 0o700) } != 0 {
			return Err(io::Error::last_os_error());
		}

		Ok(())
	}
	#[cfg(not(unix))]
	{
		std::fs::DirBuilder::new().recursive(true).create(p)
	}
}
