use std::{fs::{File, OpenOptions}, io::{self, Read, Write}, os::unix::fs::{MetadataExt, OpenOptionsExt}};

use super::Copier;

impl Copier {
	pub(super) fn copy(&mut self) -> io::Result<()> {
		let src = self.open_src()?;
		let meta = Self::file_meta(&src)?;

		let dist = self.open_dist(meta.mode())?;
		dist.set_permissions(meta.permissions()).or_else(Self::skip_attrs)?;

		self.copy_buffered(&src, &dist)?;
		self.apply_times(&dist)
	}

	fn copy_buffered(&mut self, mut src: &File, mut dist: &File) -> io::Result<()> {
		let mut buf = [0; 128 * 1024];

		loop {
			self.check()?;

			let n = match src.read(&mut buf) {
				Ok(0) => return Ok(()),
				Ok(n) => n,
				Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
				Err(e) => return Err(e),
			};

			let mut remaining = &buf[..n];
			while !remaining.is_empty() {
				self.check()?;

				let n = match dist.write(remaining) {
					Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
					Ok(n) => n,
					Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
					Err(e) => return Err(e),
				};

				remaining = &remaining[n..];
				self.progress.advance(n, &self.tx);
			}
		}
	}

	fn open_src(&self) -> io::Result<File> {
		self.check()?;
		OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK).open(&self.from)
	}

	fn open_dist(&self, mode: u32) -> io::Result<File> {
		self.check()?;

		let file = OpenOptions::new()
			.write(true)
			.create(true)
			.mode(mode)
			.custom_flags(libc::O_NONBLOCK)
			.open(&self.to)?;

		_ = Self::file_meta(&file)?; // Ensure the file is regular.
		file.set_len(0)?;

		Ok(file)
	}

	pub(super) fn reset(&mut self) -> io::Result<()> { Ok(()) }

	pub(super) fn retry(&mut self, _error: &io::Error) -> bool { false }
}
