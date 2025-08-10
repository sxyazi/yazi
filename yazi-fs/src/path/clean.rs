use std::{borrow::Cow, path::{Path, PathBuf}};

use yazi_shared::url::{Loc, Url};

pub fn clean_url<'a>(url: impl Into<Cow<'a, Url>>) -> Url {
	let cow = url.into();
	let (path, uri, urn) = clean_path_impl(&cow.loc, cow.loc.base().count(), cow.loc.trail().count());

	let loc = Loc::with(path, uri, urn).expect("Failed to create Loc from cleaned path");
	match cow {
		Cow::Borrowed(u) => Url { loc, scheme: u.scheme.clone() },
		Cow::Owned(u) => Url { loc, scheme: u.scheme },
	}
}

fn clean_path_impl(path: &Path, base: usize, trail: usize) -> (PathBuf, usize, usize) {
	use std::path::Component::*;

	let mut out = vec![];
	let mut uri_count = 0;
	let mut urn_count = 0;

	macro_rules! push {
		($i:ident, $c:ident) => {{
			out.push($c);
			if $i >= base {
				uri_count += 1;
			}
			if $i >= trail {
				urn_count += 1;
			}
		}};
	}

	for (i, c) in path.components().enumerate() {
		match c {
			CurDir => {}
			ParentDir => match out.last() {
				Some(RootDir) => {}
				Some(Normal(_)) => _ = out.pop(),
				None | Some(CurDir) | Some(ParentDir) | Some(Prefix(_)) => push!(i, c),
			},
			c => push!(i, c),
		}
	}

	(if out.is_empty() { PathBuf::from(".") } else { out.iter().collect() }, uri_count, urn_count)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_clean_url() -> anyhow::Result<()> {
		let cases = [
			// CurDir
			("archive://:3//./tmp/test.zip/foo/bar", "archive://:3//tmp/test.zip/foo/bar"),
			("archive://:3//tmp/./test.zip/foo/bar", "archive://:3//tmp/test.zip/foo/bar"),
			("archive://:3//tmp/./test.zip/./foo/bar", "archive://:3//tmp/test.zip/foo/bar"),
			("archive://:3//tmp/./test.zip/./foo/./bar/.", "archive://:3//tmp/test.zip/foo/bar"),
			// ParentDir
			("archive://:3:2//../../tmp/test.zip/foo/bar", "archive://:3:2//tmp/test.zip/foo/bar"),
			("archive://:3:2//tmp/../../test.zip/foo/bar", "archive://:3:2//test.zip/foo/bar"),
			("archive://:4:2//tmp/test.zip/../../foo/bar", "archive://:2:2//foo/bar"),
			("archive://:5:2//tmp/test.zip/../../foo/bar", "archive://:3:2//foo/bar"),
			("archive://:4:4//tmp/test.zip/foo/bar/../../", "archive://:1:1//tmp/test.zip"),
			("archive://:5:4//tmp/test.zip/foo/bar/../../", "archive://:2:1//tmp/test.zip"),
			("archive://:4:4//tmp/test.zip/foo/bar/../../../", "archive:////tmp"),
		];

		for (input, expected) in cases {
			let input: Url = input.parse()?;
			#[cfg(unix)]
			assert_eq!(format!("{:?}", clean_url(input)), expected);
			#[cfg(windows)]
			assert_eq!(format!("{:?}", clean_url(input)).replace(r"\", "/"), expected.replace(r"\", "/"));
		}
		Ok(())
	}
}
