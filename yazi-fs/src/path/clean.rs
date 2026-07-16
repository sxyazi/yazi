use std::{iter, sync::Arc};

use yazi_shared::{auth::{Auth, AuthKind}, path::{DynPath, PathBufDyn, PathDyn}, url::{UrlBuf, UrlCow, UrlLike}};

pub fn clean_url<'a>(url: impl Into<UrlCow<'a>>) -> UrlBuf {
	let cow: UrlCow = url.into();

	let depth = cow.loc().components().count();
	let base = depth - cow.uri().components().count();
	let trail = depth - cow.urn().components().count();

	let (mut spec, path) = cow.into_pair();
	let (path, uri, urn, auth) = clean_path_impl(
		path.dyn_path(),
		base,
		trail,
		(spec.kind == AuthKind::Hub).then(|| spec.auth.clone()),
	);

	spec.auth = auth.unwrap_or(spec.auth);
	(spec.with_ports(uri, urn), path).try_into().expect("UrlBuf from cleaned path")
}

fn clean_path_impl(
	path: PathDyn,
	base: usize,
	trail: usize,
	auth: Option<Arc<Auth>>,
) -> (PathBufDyn, usize, usize, Option<Arc<Auth>>) {
	use yazi_shared::path::Component::*;

	let mut auths: Vec<_> = iter::successors(auth, |auth| auth.parent.clone()).collect();
	let root = auths.pop();

	let mut out = vec![];
	let mut uri_count = 0;
	let mut urn_count = 0;

	macro_rules! push {
		($i:ident, $c:ident, $auth:ident) => {{
			out.push(($i, $c, $auth));
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
			if let Some((i, ..)) = out.pop() {
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
		let auth = (root.is_some() && c.has_auth()).then(|| auths.pop().unwrap());
		match c {
			CurDir => {}
			ParentDir => match out.last().map(|(_, c, _)| c) {
				Some(RootDir) => {}
				Some(Normal(_)) => pop!(),
				None | Some(CurDir) | Some(ParentDir) | Some(Prefix(_)) => push!(i, c, auth),
			},
			c => push!(i, c, auth),
		}
	}

	let kind = path.kind();
	let path = if out.is_empty() {
		PathBufDyn::with_str(kind, ".")
	} else {
		PathBufDyn::from_components(kind, out.iter().map(|(_, c, _)| *c))
			.expect("components with same kind")
	};

	let auth = root.map(|mut parent| {
		for mut auth in out.into_iter().filter_map(|(_, _, auth)| auth) {
			Arc::make_mut(&mut auth).parent = Some(parent);
			parent = auth;
		}
		parent
	});
	(path, uri_count, urn_count, auth)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_clean_url() -> anyhow::Result<()> {
		yazi_shared::init_tests();
		let cases = [
			// CurDir
			("test-mount://7z:3//./tmp/test.zip/foo/bar", "test-mount://7z:3//tmp/test.zip/foo/bar"),
			("test-mount://7z:3//tmp/./test.zip/foo/bar", "test-mount://7z:3//tmp/test.zip/foo/bar"),
			("test-mount://7z:3//tmp/./test.zip/./foo/bar", "test-mount://7z:3//tmp/test.zip/foo/bar"),
			(
				"test-mount://7z:3//tmp/./test.zip/./foo/./bar/.",
				"test-mount://7z:3//tmp/test.zip/foo/bar",
			),
			// ParentDir
			(
				"test-mount://7z:3:2//../../tmp/test.zip/foo/bar",
				"test-mount://7z:3:2//tmp/test.zip/foo/bar",
			),
			("test-mount://7z:3:2//tmp/../../test.zip/foo/bar", "test-mount://7z:3:2//test.zip/foo/bar"),
			("test-mount://7z:4:2//tmp/test.zip/../../foo/bar", "test-mount://7z:2:2//foo/bar"),
			("test-mount://7z:5:2//tmp/test.zip/../../foo/bar", "test-mount://7z:2:2//foo/bar"),
			("test-mount://7z:4:4//tmp/test.zip/foo/bar/../../", "test-mount://7z//tmp/test.zip"),
			("test-mount://7z:5:4//tmp/test.zip/foo/bar/../../", "test-mount://7z:1//tmp/test.zip"),
			("test-mount://7z:4:4//tmp/test.zip/foo/bar/../../../", "test-mount://7z//tmp"),
			("sftp://vps//root/.config/yazi/../../Downloads", "sftp://vps//root/Downloads"),
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

	#[test]
	fn test_clean_hub() -> anyhow::Result<()> {
		yazi_shared::init_tests();

		for (input, expected) in [
			("test-hub://dotdot/@foo,root/foo/..", "test-hub://root/@/."),
			("test-hub://bar/@dotdot,foo,root/foo/../bar", "test-hub://bar/@root/bar"),
			("test-hub://dotdot/@root//..", "test-hub://root/@//"),
			("test-hub://bar/@dotdot,foo,root//foo/../bar", "test-hub://bar/@root//bar"),
			("test-hub://d3/@d2,d1,root/../../..", "test-hub://d3/@d2,d1,root/../../.."),
		] {
			let input: UrlBuf = input.parse()?;
			#[cfg(unix)]
			assert_eq!(format!("{:?}", clean_url(input)), expected);
			#[cfg(windows)]
			assert_eq!(format!("{:?}", clean_url(input)).replace(r"\", "/"), expected);
		}
		Ok(())
	}
}
