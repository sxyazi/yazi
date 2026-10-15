use std::{ffi::{CString, c_char, c_void}, fs::{self, File, OpenOptions, Permissions}, io, mem::ManuallyDrop, os::{fd::AsRawFd, macos::fs::MetadataExt as _, unix::{ffi::OsStrExt, fs::{MetadataExt, OpenOptionsExt}}}};

use scopeguard::guard;
use yazi_shim::nonneg_ok;

use super::{Copier, Copystate, Decmpfs};

impl Copier {
	pub(super) fn copy(&mut self) -> io::Result<()> {
		let src = self.open_src()?;
		let meta = Self::file_meta(&src)?;

		if self.try_clone(&src, meta.len())? {
			return Ok(());
		}

		// New compressed targets need owner access, restore source permissions after copying.
		let compressed = meta.st_flags() & libc::UF_COMPRESSED != 0;
		let dist = self.open_dist(if compressed { 0o600 } else { meta.mode() })?;

		if compressed {
			Decmpfs::new(&src, &dist, &meta).copy(self)?;
		} else {
			self.copy_xattrs(&src, &dist)?;
			self.copy_data(&src, &dist)?;
			self.copy_security(&src, &dist, meta.permissions())?;
		}

		self.apply_times(&dist)
	}

	fn try_clone(&mut self, src: &File, len: u64) -> io::Result<bool> {
		self.check()?;

		let path = CString::new(self.to.as_os_str().as_bytes())?;
		let result =
			nonneg_ok(unsafe { libc::fclonefileat(src.as_raw_fd(), libc::AT_FDCWD, path.as_ptr(), 0) });

		match result {
			Ok(()) => {
				self.progress.update(len);
				if let Ok(times) = self.attrs.try_into() {
					fs::set_times(&self.to, times).ok();
				}

				Ok(true)
			}
			// The filesystem does not support cloning: copy the data instead.
			Err(e) if e.raw_os_error() == Some(libc::ENOTSUP) => Ok(false),
			// Cloning requires a new destination: open and truncate the existing file instead.
			Err(e) if e.raw_os_error() == Some(libc::EEXIST) => Ok(false),
			// Cloning cannot cross filesystems: copy the data between them instead.
			Err(e) if e.raw_os_error() == Some(libc::EXDEV) => Ok(false),
			// Cloning is prohibited: try ordinary copying under its own permission checks.
			Err(e) if e.raw_os_error() == Some(libc::EPERM) => Ok(false),
			// Clone access was denied: let ordinary copying determine whether it can proceed.
			Err(e) if e.raw_os_error() == Some(libc::EACCES) => Ok(false),
			Err(e) => Err(e),
		}
	}

	pub(super) fn copy_xattrs(&mut self, src: &File, dist: &File) -> io::Result<bool> {
		// Xattr callbacks can promote ignored native errors to failures; leave this stage without a callback.
		loop {
			self.check()?;

			match nonneg_ok(unsafe {
				libc::fcopyfile(src.as_raw_fd(), dist.as_raw_fd(), *self.state, libc::COPYFILE_XATTR)
			}) {
				// An attribute may disappear during copying: retry ENOATTR with fresh metadata.
				Err(e) if e.raw_os_error() == Some(libc::ENOATTR) && self.retry(&e) => self.reset()?,
				Ok(()) => return Ok(true),
				Err(e) => {
					Self::skip_attrs(e)?;
					// Failed calls can cache incomplete metadata; start DATA and SECURITY with a fresh state.
					self.reset()?;
					return Ok(false);
				}
			}
		}
	}

	pub(super) fn copy_data(&mut self, src: &File, dist: &File) -> io::Result<u64> {
		self.check()?;

		for (key, value) in [
			(libc::COPYFILE_STATE_STATUS_CB, callback as *const c_void),
			(libc::COPYFILE_STATE_STATUS_CTX, (self as *mut Self).cast()),
		] {
			nonneg_ok(unsafe { libc::copyfile_state_set(*self.state, key as _, value) })?;
		}

		let result = nonneg_ok(unsafe {
			libc::fcopyfile(src.as_raw_fd(), dist.as_raw_fd(), *self.state, libc::COPYFILE_DATA)
		});

		self.error.take().map_or(result, Err)?;
		let bytes = self.state.copied()?;

		self.progress.update(bytes);
		Ok(bytes)
	}

	pub(super) fn copy_security(
		&mut self,
		src: &File,
		dist: &File,
		perm: Permissions,
	) -> io::Result<()> {
		// Data is already copied: retry SECURITY alone and ignore PermissionDenied or Unsupported attribute errors.
		loop {
			self.check()?;

			let result = nonneg_ok(unsafe {
				libc::fcopyfile(src.as_raw_fd(), dist.as_raw_fd(), *self.state, libc::COPYFILE_SECURITY)
			});

			match self.error.take().map_or(result, Err) {
				Err(e) if e.raw_os_error() == Some(libc::ENOATTR) && self.retry(&e) => {}
				Ok(()) => return Ok(()),
				Err(e) => break Self::skip_attrs(e)?,
			}
		}

		self.set_permissions(dist, perm)
	}

	pub(super) fn set_permissions(&self, dist: &File, perm: Permissions) -> io::Result<()> {
		self.check()?;
		dist.set_permissions(perm).or_else(Self::skip_attrs)
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

	pub(super) fn retry(&mut self, error: &io::Error) -> bool {
		if self.retries < 3 && matches!(error.raw_os_error(), Some(libc::EPERM | libc::ENOATTR)) {
			self.retries += 1;
			true
		} else {
			false
		}
	}

	pub(super) fn reset(&mut self) -> io::Result<()> {
		self.state = Copystate::new()?;
		self.error = None;

		Ok(())
	}
}

extern "C" fn callback(
	what: i32,
	stage: i32,
	// The native caller owns this state; suppress Drop while preserving its pointer ABI.
	state: ManuallyDrop<Copystate>,
	_src: *const c_char,
	_dst: *const c_char,
	data: *mut c_void,
) -> i32 {
	let copier = unsafe { &mut *data.cast::<Copier>() };
	let errno = guard(unsafe { *libc::__error() }, |errno| unsafe { *libc::__error() = errno });

	let result = (|| {
		copier.check()?;

		if what == libc::COPYFILE_COPY_XATTR {
			// Saving ignored quarantine errors would turn native success into failure; let fcopyfile decide which xattr errors are fatal.
			Ok(libc::COPYFILE_CONTINUE)
		} else if stage == libc::COPYFILE_ERR {
			// Recoverable errors need not abort copying; let retry() decide whether native recovery may continue, otherwise propagate the error.
			let error = io::Error::from_raw_os_error(*errno);
			if copier.retry(&error) { Ok(libc::COPYFILE_CONTINUE) } else { Err(error) }
		} else if what == libc::COPYFILE_COPY_DATA && stage == libc::COPYFILE_PROGRESS {
			copier.progress.update(state.copied()?);
			copier.progress.report(&copier.tx);
			Ok(libc::COPYFILE_CONTINUE)
		} else {
			Ok(libc::COPYFILE_CONTINUE)
		}
	})();

	match result {
		Ok(action) => action,
		Err(e) => {
			copier.error = Some(e);
			libc::COPYFILE_QUIT
		}
	}
}
