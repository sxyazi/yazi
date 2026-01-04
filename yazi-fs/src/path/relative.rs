use anyhow::{Result, bail};
use yazi_shared::path::{PathBufDyn, PathCow, PathDyn, PathLike};

pub fn path_relative_to<'a, 'b, P, Q>(from: P, to: Q) -> Result<PathCow<'b>>
where
	P: Into<PathCow<'a>>,
	Q: Into<PathCow<'b>>,
{
	path_relative_to_impl(from.into(), to.into())
}

fn path_relative_to_impl<'a>(from: PathCow<'_>, to: PathCow<'a>) -> Result<PathCow<'a>> {
	use yazi_shared::path::Component::*;

	if from.is_absolute() != to.is_absolute() {
		return if to.is_absolute() {
			Ok(to)
		} else {
			bail!("Paths must be both absolute or both relative: {from:?} and {to:?}");
		};
	}

	if from == to {
		return Ok(PathDyn::with_str(from.kind(), ".").into());
	}

	let (mut f_it, mut t_it) = (from.components(), to.components());
	let (f_head, t_head) = loop {
		match (f_it.next(), t_it.next()) {
			(Some(RootDir), Some(RootDir)) => {}
			(Some(Prefix(a)), Some(Prefix(b))) if a == b => {}
			(Some(Prefix(_) | RootDir), _) | (_, Some(Prefix(_) | RootDir)) => {
				return Ok(to);
			}
			(None, None) => break (None, None),
			(a, b) if a != b => break (a, b),
			_ => (),
		}
	};

	let dots = f_head.into_iter().chain(f_it).map(|_| ParentDir);
	let rest = t_head.into_iter().chain(t_it);

	let buf = PathBufDyn::from_components(from.kind(), dots.chain(rest))?;
	Ok(buf.into())
}

#[cfg(test)]
mod tests {
	use yazi_shared::path::PathDyn;

	use super::*;

	#[test]
	fn test_path_relative_to() {
		yazi_shared::init_tests();

		#[cfg(unix)]
		let cases = [
			// Same paths
			("", "", "."),
			(".", ".", "."),
			("/", "/", "."),
			("/a", "/a", "."),
			// Relative paths
			("foo", "bar", "../bar"),
			// Absolute paths
			("/a/b", "/a/b/c", "c"),
			("/a/b/c", "/a/b", ".."),
			("/a/b/d", "/a/b/c", "../c"),
			("/a/b/c", "/a", "../.."),
			("/a/b/c/", "/a/d/", "../../d"),
			("/a/b/b", "/a/a/b", "../../a/b"),
		];

		#[cfg(windows)]
		let cases = [
			(r"C:\a\b", r"C:\a\b\c", "c"),
			(r"C:\a\b\c", r"C:\a\b", r".."),
			(r"C:\a\b\d", r"C:\a\b\c", r"..\c"),
			(r"C:\a\b\c", r"C:\a", r"..\.."),
			(r"C:\a\b\b", r"C:\a\a\b", r"..\..\a\b"),
		];

		for (from, to, expected) in cases {
			let from = PathDyn::Os(from.as_ref());
			let to = PathDyn::Os(to.as_ref());
			assert_eq!(path_relative_to(from, to).unwrap().to_str().unwrap(), expected);
		}
	}
}
