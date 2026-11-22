use std::{borrow::Cow, io::{self, ErrorKind}, path::{Path, PathBuf}};

pub struct Symlinker;

impl Symlinker {
	pub fn temporary_name() -> String { format!(".yazilink-{:016x}", rand::random::<u64>()) }

	pub(crate) fn temporary_link<F>(link: &Path, create: F) -> io::Result<PathBuf>
	where
		F: Fn(&Path) -> io::Result<()>,
	{
		let mut retries = 7;
		loop {
			let tmp = link.with_file_name(Self::temporary_name());
			match create(&tmp) {
				Ok(()) => return Ok(tmp),
				Err(e) if e.kind() == ErrorKind::AlreadyExists && retries > 0 => retries -= 1,
				Err(e) => return Err(e),
			}
		}
	}

	pub(super) fn resolve_target<'a>(original: &'a Path, link: &Path) -> Cow<'a, Path> {
		if original.is_absolute() {
			Cow::Borrowed(original)
		} else {
			Cow::Owned(link.parent().unwrap_or(Path::new(".")).join(original))
		}
	}
}
