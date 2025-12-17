use yazi_shared::{path::{PathBufDyn, PathDyn}, url::{UrlBuf, UrlCow, UrlLike}};

pub fn clean_url<'a>(url: impl Into<UrlCow<'a>>) -> UrlBuf {
	let cow: UrlCow = url.into();
	let (path, uri, urn) = clean_path_impl(
		cow.loc(),
		cow.base().components().count() - 1,
		cow.trail().components().count() - 1,
	);

	let scheme = cow.into_scheme().into_owned().with_ports(uri, urn);
	(scheme, path).try_into().expect("UrlBuf from cleaned path")
}

fn clean_path_impl(path: PathDyn, base: usize, trail: usize) -> (PathBufDyn, usize, usize) {
	use yazi_shared::path::Component::*;

	let mut out = vec![];
	let mut uri_count = 0;
	let mut urn_count = 0;

	macro_rules! push {
		($i:ident, $c:ident) => {{
			out.push(($i, $c));
			if $i >= base {
				uri_count += 1;
			}
			if $i >= trail {
				urn_count += 1;
			}
		}};
	}

	macro_rules! pop {
		() => {{
			if let Some((i, _)) = out.pop() {
				if i >= base {
					uri_count -= 1;
				}
				if i >= trail {
					urn_count -= 1;
				}
			}
		}};
	}

	for (i, c) in path.components().enumerate() {
		match c {
			CurDir => {}
			ParentDir => match out.last().map(|(_, c)| c) {
				Some(RootDir) => {}
				Some(Normal(_)) => pop!(),
				None | Some(CurDir) | Some(ParentDir) | Some(Prefix(_)) => push!(i, c),
			},
			c => push!(i, c),
		}
	}

	let kind = path.kind();
	let path = if out.is_empty() {
		PathBufDyn::with_str(kind, ".")
	} else {
		PathBufDyn::from_components(kind, out.into_iter().map(|(_, c)| c))
			.expect("components with same kind")
	};

	(path, uri_count, urn_count)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_clean_url() -> anyhow::Result<()> {
		yazi_shared::init_tests();
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
			("archive://:5:2//tmp/test.zip/../../foo/bar", "archive://:2:2//foo/bar"),
			("archive://:4:4//tmp/test.zip/foo/bar/../../", "archive:////tmp/test.zip"),
			("archive://:5:4//tmp/test.zip/foo/bar/../../", "archive://:1//tmp/test.zip"),
			("archive://:4:4//tmp/test.zip/foo/bar/../../../", "archive:////tmp"),
			("sftp://test//root/.config/yazi/../../Downloads", "sftp://test//root/Downloads"),
		];

		for (input, expected) in cases {
			let input: UrlBuf = input.parse()?;
			#[cfg(unix)]
			assert_eq!(format!("{:?}", clean_url(input)), expected);
			#[cfg(windows)]
			assert_eq!(format!("{:?}", clean_url(input)).replace(r"\", "/"), expected.replace(r"\", "/"));
		}
		Ok(())
	}
}
