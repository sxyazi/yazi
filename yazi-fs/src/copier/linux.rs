use std::{fs::{File, OpenOptions}, io::{self, Read, Write}, os::unix::fs::{MetadataExt, OpenOptionsExt}, ptr, sync::atomic::{AtomicBool, AtomicU8, Ordering}, thread, time::Duration};

use rustix::io::Errno;

use super::Copier;

// Copy in 32 MiB chunks to avoid blocking the kernel for too long on large files.
const COPY_CHUNK: usize = 32 * 1024 * 1024;

// 0 = not probed, 1 = available, 2 = unavailable.
static COPY_RANGE: AtomicU8 = AtomicU8::new(0);

impl Copier {
	pub(super) fn copy(&mut self) -> io::Result<()> {
		let src = self.open_src()?;
		let meta = Self::file_meta(&src)?;

		let dist = self.open_dist(meta.mode())?;
		dist.set_permissions(meta.permissions()).or_else(Self::skip_attrs)?;

		// Virtual files may report zero length while still containing data: use buffered reads for them.
		if meta.len() == 0 {
			self.copy_buffered(&src, &dist)
		} else if self.copy_range(&src, &dist)? {
			Ok(())
		} else if self.copy_sendfile(&src, &dist)? {
			Ok(())
		} else {
			self.copy_buffered(&src, &dist)
		}?;

		self.apply_times(&dist)
	}

	fn copy_range(&mut self, src: &File, dist: &File) -> io::Result<bool> {
		let mut avail = COPY_RANGE.load(Ordering::Relaxed);
		if avail == 2 {
			return Ok(false);
		}

		let start = self.progress.written();
		loop {
			self.check()?;

			let result = rustix::fs::copy_file_range(src, None, dist, None, COPY_CHUNK);
			if avail == 0 && result.is_ok() {
				avail = 1;
				COPY_RANGE.store(avail, Ordering::Relaxed);
			}

			match result {
				Ok(0) => return Ok(self.progress.written() > start),
				Ok(n) => self.progress.advance(n, &self.tx),
				Err(e) if e == Errno::OVERFLOW => return Ok(false),
				Err(e) if self.progress.written() == start && Self::copy_range_fallback(e, avail) => {
					return Ok(false);
				}
				Err(e) => return Err(e.into()),
			}
		}
	}

	fn copy_range_fallback(error: Errno, avail: u8) -> bool {
		let probe = match error {
			// The syscall may be missing or intercepted; probe before caching its availability.
			Errno::NOSYS => true,
			// File restrictions and seccomp can both deny copying; probe to distinguish them before caching availability.
			Errno::PERM => true,
			// These files or offsets are unsupported; fall back without disabling the syscall globally.
			Errno::INVAL => false,
			// Range copying cannot handle this filesystem pair; fall back while keeping the syscall available.
			Errno::XDEV => false,
			// The filesystem or kernel may reject range copying; probe before deciding whether to disable it globally.
			Errno::OPNOTSUPP => true,
			// Descriptor access modes or flags may reject range copying; fall back while keeping the syscall available.
			Errno::BADF => false,
			_ => return false,
		};

		if avail == 0 {
			let b = if probe { Self::copy_range_probe() } else { true };
			COPY_RANGE.store(if b { 1 } else { 2 }, Ordering::Relaxed);
		}

		true
	}

	fn copy_range_probe() -> bool {
		// Invalid handles (-1) distinguish syscall restrictions from failures specific to the copied files.
		unsafe {
			libc::syscall(
				libc::SYS_copy_file_range,
				-1,
				ptr::null::<libc::loff_t>(),
				-1,
				ptr::null::<libc::loff_t>(),
				1usize,
				0u32,
			);
		}

		io::Error::last_os_error().raw_os_error() == Some(libc::EBADF)
	}

	fn copy_sendfile(&mut self, src: &File, dist: &File) -> io::Result<bool> {
		static AVAILABLE: AtomicBool = AtomicBool::new(true);

		if !AVAILABLE.load(Ordering::Relaxed) {
			return Ok(false);
		}

		let start = self.progress.written();
		loop {
			self.check()?;

			match rustix::fs::sendfile(dist, src, None, COPY_CHUNK) {
				Ok(0) => return Ok(true),
				Ok(n) => self.progress.advance(n, &self.tx),
				Err(e) if e == Errno::OVERFLOW => return Ok(false),
				Err(e) if self.progress.written() == start => match e {
					Errno::NOSYS | Errno::PERM => {
						AVAILABLE.store(false, Ordering::Relaxed);
						return Ok(false);
					}
					Errno::INVAL => return Ok(false),
					_ => return Err(e.into()),
				},
				Err(e) => return Err(e.into()),
			}
		}
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
		let mut options = OpenOptions::new();
		options.read(true).custom_flags(libc::O_NONBLOCK);

		loop {
			self.check()?;

			match options.open(&self.from) {
				// O_NONBLOCK avoids hanging on FIFOs but makes leases return WouldBlock: retry while checking cancellation.
				Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
					thread::sleep(Duration::from_millis(50));
				}
				result => return result,
			}
		}
	}

	fn open_dist(&self, mode: u32) -> io::Result<File> {
		let mut options = OpenOptions::new();
		options.write(true).create(true).mode(mode).custom_flags(libc::O_NONBLOCK);

		let file = loop {
			self.check()?;

			match options.open(&self.to) {
				// O_NONBLOCK avoids hanging on FIFOs but makes leases return WouldBlock: retry while checking cancellation.
				Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
					thread::sleep(Duration::from_millis(50));
				}
				result => break result?,
			}
		};

		// Linux ftruncate rejects non-regular files: truncate here without a separate metadata check.
		file.set_len(0)?;

		Ok(file)
	}

	pub(super) fn reset(&mut self) -> io::Result<()> { Ok(()) }

	pub(super) fn retry(&mut self, _error: &io::Error) -> bool { false }
}
