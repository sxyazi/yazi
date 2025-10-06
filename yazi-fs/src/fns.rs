use tokio::io;
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
