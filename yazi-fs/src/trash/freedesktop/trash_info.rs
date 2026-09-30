use std::{borrow::Cow, ffi::OsStr, fs::File, io::{self, BufRead, BufReader}, os::unix::ffi::OsStrExt, path::{Path, PathBuf}, str::from_utf8, time::SystemTime};

use percent_encoding::percent_decode;
use yazi_shim::{Uzers, path::PathExt};

pub(super) struct TrashInfo {
	pub(super) root:     PathBuf,
	pub(super) backing:  PathBuf,
	pub(super) original: PathBuf,
	pub(super) dtime: Option<SystemTime>,
}

impl TrashInfo {
	// Parses from a trashinfo path, e.g.:
	//   /home/alice/.local/share/Trash/info/cat.jpg.trashinfo
	pub(super) fn parse(info: &Path) -> io::Result<Self> {
		if info.extension() != Some(OsStr::new("trashinfo")) {
			return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid trash info path"));
		}

		// /home/alice/.local/share/Trash
		let root = info
			.parent()
			.filter(|p| p.file_name() == Some(OsStr::new("info")))
			.and_then(Path::parent)
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid trash info path"))?;

		// cat.jpg
		let stem = info
			.file_stem()
			.filter(|&stem| stem != OsStr::new(".") && stem != OsStr::new(".."))
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid trash info path"))?;

		let (original, dtime) = Self::parse_original(info, root)?;
		if original.file_name().is_none() {
			return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid original trash path"));
		}

		Ok(Self { root: root.to_owned(), backing: root.join("files").join(stem), original, dtime, })
	}

	fn parse_original(info: &Path, root: &Path) -> io::Result<(PathBuf, Option<SystemTime>)> {
		let mut reader = BufReader::new(File::open(info)?);
		let mut line = Vec::new();

		reader.read_until(b'\n', &mut line)?;
		Self::trim_line(&mut line);
		if line != b"[Trash Info]" {
			return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid trash info header"));
		}

		let mut parsed_path: Option<PathBuf> = None;
		let mut parsed_dtime: Option<SystemTime> = None;
		
		loop {
			line.clear();
			if reader.read_until(b'\n', &mut line)? == 0 {
				break;
			}

			Self::trim_line(&mut line);

			if let Some(value) = line.strip_prefix(b"Path=") {
			    let decoded: Cow<[u8]> = percent_decode(value).into();
			    let path = Path::new(OsStr::from_bytes(decoded.as_ref()));
				if path.as_os_str().is_empty() || !path.is_absolute() && path.has_parent_component() {
					break;
				}
				parsed_path = Some(if path.is_absolute() {
				    path.to_owned()
				} else {
				    Self::mount_point(root)?.join(path)
				})
			} else if let Some(value) = line.strip_prefix(b"DeletionDate=") {
			    parsed_dtime = Self::parse_deletion_date(value);
			}

			if parsed_dtime.is_some() && parsed_path.is_some() {
			    break;
			}
		}
		let final_path = parsed_path.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "trash info has no Path"))?;
		Ok((final_path, parsed_dtime))
	}

	fn parse_deletion_date(bytes: &[u8]) -> Option<SystemTime> {
	    let s = from_utf8(bytes).ok()?;
		let dt = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok()?;

		Some(dt.and_utc().into())
	}

	// /mnt/disk/.Trash/1000           =>  /mnt/disk
	// /home/alice/.local/share/Trash  =>  /home/alice/.local/share
	fn mount_point(root: &Path) -> io::Result<&Path> {
		let uid = Uzers::uid().to_string();

		if root.file_name() == Some(OsStr::new(&uid))
			&& let Some(parent) = root.parent()
			&& parent.file_name() == Some(OsStr::new(".Trash"))
		{
			parent.parent()
		} else {
			root.parent()
		}
		.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid trash mount point"))
	}

	fn trim_line(line: &mut Vec<u8>) {
		while matches!(line.last(), Some(b'\n' | b'\r')) {
			line.pop();
		}
	}
}
