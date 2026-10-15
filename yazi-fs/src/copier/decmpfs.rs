use std::{fs::{File, Metadata}, io, mem, os::{fd::AsRawFd, macos::fs::MetadataExt as _}};

use scopeguard::guard;
use strum::EnumIs;
use yazi_ffi::{BsdFlags, Xattr};

use super::Copier;

/// Copies transparent compression between pinned file handles.
pub struct Decmpfs<'a> {
	src:  &'a File,
	dist: &'a File,
	meta: &'a Metadata,
}

impl<'a> Decmpfs<'a> {
	/// Use metadata from the source handle and an opened, truncated destination writable for xattrs and forks.
	pub(super) fn new(src: &'a File, dist: &'a File, meta: &'a Metadata) -> Self {
		Self { src, dist, meta }
	}

	pub(super) fn copy(&self, copy: &mut Copier) -> io::Result<()> {
		if !copy.copy_xattrs(self.src, self.dist)? {
			return self.copy_fallback(copy, Ok(false));
		}

		copy.check()?;
		if !Self::is_supported(self.src) || !Self::is_supported(self.dist) {
			return self.copy_fallback(copy, Ok(true));
		}

		let header = match Xattr::new(self.src, libc::XATTR_SHOWCOMPRESSION).get(c"com.apple.decmpfs") {
			Ok(header) => header,
			// Header reads only decide whether compression can be preserved; copy ordinary DATA on failure, as Apple does.
			Err(_) => return self.copy_fallback(copy, Ok(true)),
		};

		let fork = Fork::parse(&header, self.meta.len());
		if fork.is_unsupported() {
			return self.copy_fallback(copy, Ok(true));
		} else if fork.is_resource() {
			let outcome = match ResourceFork::new(self.src, self.dist) {
				Ok(Some(resource)) => resource.write(self.meta.len(), copy)?,
				Ok(None) => return self.copy_fallback(copy, Ok(true)),
				Err(e) => return self.copy_fallback(copy, Err(e)),
			};

			match outcome {
				ForkOut::Complete => {}
				ForkOut::Fallback(reason) => {
					self.deactivate();
					return self.copy_fallback(copy, reason);
				}
			}
		}

		match self.activate(copy, &header) {
			// Compression was activated; report logical completion before final SECURITY.
			Ok(true) => {
				copy.progress.update(self.meta.len());
				copy.copy_security(self.src, self.dist, self.meta.permissions())
			}
			// The filesystem did not activate compression; continue to cleanup and ordinary DATA copying.
			Ok(false) => {
				self.deactivate();
				self.copy_fallback(copy, Ok(true))
			}
			// Only ignorable attribute errors permit DATA fallback; propagate other errors before cleanup.
			Err(e) => {
				Copier::skip_attrs(e)?;
				self.deactivate();
				self.copy_fallback(copy, Ok(false))
			}
		}
	}

	fn copy_fallback(&self, copy: &mut Copier, reason: io::Result<bool>) -> io::Result<()> {
		let security = match reason {
			// The caller already chose the attribute policy for this fallback; preserve that choice.
			Ok(b) => b,
			// Preserve the existing recovery policy for missing files; macOS ENOATTR does not map to NotFound.
			Err(e) if e.kind() == io::ErrorKind::NotFound => true,
			// Preparation may leave compression incomplete; propagate fatal errors, otherwise discard the prior XATTR state and skip SECURITY.
			Err(e) => {
				Copier::skip_attrs(e)?;
				copy.reset()?;
				false
			}
		};

		copy.copy_data(self.src, self.dist)?;

		if security {
			copy.copy_security(self.src, self.dist, self.meta.permissions())
		} else {
			copy.set_permissions(self.dist, self.meta.permissions())
		}
	}

	fn activate(&self, copy: &Copier, header: &[u8]) -> io::Result<bool> {
		copy.check()?;

		Xattr::new(self.dist, libc::XATTR_SHOWCOMPRESSION).set(c"com.apple.decmpfs", header)?;

		// Activate before applying source ACLs and flags, so failed compression can still be cleaned up.
		if BsdFlags::set(self.dist, libc::UF_COMPRESSED, u32::MAX).is_err() {
			return Ok(false);
		}

		Ok(self.dist.metadata().is_ok_and(|meta| meta.st_flags() & libc::UF_COMPRESSED != 0))
	}

	fn deactivate(&self) {
		let xattr = Xattr::new(self.dist, libc::XATTR_SHOWCOMPRESSION);
		xattr.remove(c"com.apple.decmpfs").ok();
		xattr.remove(c"com.apple.ResourceFork").ok();
	}

	fn is_supported(file: &File) -> bool {
		#[repr(C)]
		struct Capabilities {
			length: u32,
			volume: libc::vol_capabilities_attr_t,
		}

		let mut fs: libc::statfs = unsafe { mem::zeroed() };
		if unsafe { libc::fstatfs(file.as_raw_fd(), &mut fs) } != 0 {
			return false;
		}

		let mut attrs = libc::attrlist {
			bitmapcount: libc::ATTR_BIT_MAP_COUNT as _,
			volattr: libc::ATTR_VOL_CAPABILITIES,
			..unsafe { mem::zeroed() }
		};
		let mut caps: Capabilities = unsafe { mem::zeroed() };
		let result = unsafe {
			libc::getattrlist(
				fs.f_mntonname.as_ptr(),
				(&raw mut attrs).cast(),
				(&raw mut caps).cast(),
				mem::size_of_val(&caps),
				0,
			)
		};

		let format = libc::VOL_CAPABILITIES_FORMAT as usize;
		result == 0
			&& caps.volume.valid[format] & libc::VOL_CAP_FMT_DECMPFS_COMPRESSION != 0
			&& caps.volume.capabilities[format] & libc::VOL_CAP_FMT_DECMPFS_COMPRESSION != 0
	}
}

// --- Fork
#[derive(EnumIs)]
enum Fork {
	Unsupported,
	Inline,
	Resource,
}

impl Fork {
	fn parse(header: &[u8], logical_size: u64) -> Self {
		if header.len() < 16 || &header[..4] != b"fpmc" {
			return Self::Unsupported;
		}

		if u64::from_le_bytes(header[8..16].try_into().unwrap()) != logical_size {
			return Self::Unsupported;
		}

		match u32::from_le_bytes(header[4..8].try_into().unwrap()) {
			// Types 3=zlib, 7=LZVN, 9=uncompressed, 11=LZFSE, 13=LZBITMAP store their payload in the decmpfs xattr.
			// Copying that xattr transfers all data, so no resource fork needs copying.
			3 | 7 | 9 | 11 | 13 => Self::Inline,

			// Types 4=zlib, 8=LZVN, 10=uncompressed, 12=LZFSE, 14=LZBITMAP store the payload in a resource fork.
			// Copy the fork in chunks to preserve its format with progress and cancellation between chunks.
			4 | 8 | 10 | 12 | 14 => Self::Resource,

			// Unknown compression formats must use ordinary reads, as Apple's copyfile does.
			_ => Self::Unsupported,
		}
	}
}

// --- ResourceFork
struct ResourceFork<'a> {
	src:  Xattr<'a>,
	dist: Xattr<'a>,
	size: u64,
}

impl<'a> ResourceFork<'a> {
	fn new(src: &'a File, dist: &'a File) -> io::Result<Option<Self>> {
		let src = Xattr::new(src, libc::XATTR_SHOWCOMPRESSION);
		let dist = Xattr::new(dist, libc::XATTR_SHOWCOMPRESSION);

		let size = match src.read_at(c"com.apple.ResourceFork", &mut [], 0) {
			Ok(size) if let Ok(size) = u32::try_from(size) => size as u64,
			// The xattr position API is 32-bit; larger forks use ordinary DATA and still copy SECURITY.
			Ok(_) => return Ok(None),
			// Like copyfile, a failed size query skips this fork rather than failing attribute copying.
			Err(_) => return Ok(None),
		};

		Ok(Some(Self { src, dist, size }))
	}

	fn write(&self, logical_size: u64, copy: &mut Copier) -> io::Result<ForkOut> {
		copy.check()?;

		copy.progress.scale(logical_size, self.size);
		let copy = &mut **guard(copy, |copy| copy.progress.unscale());

		let (mut buf, mut written) = ([0; 128 * 1024], 0);
		loop {
			copy.check()?;

			// A fork can grow after sizing; fall back instead of wrapping its next read position.
			let Ok(pos) = u32::try_from(written) else { return Ok(ForkOut::Fallback(Ok(true))) };
			let size = match self.src.read_at(c"com.apple.ResourceFork", &mut buf, pos) {
				// A changed fork size cannot prove the payload is complete; copy ordinary DATA instead.
				Ok(0) if written != self.size => return Ok(ForkOut::Fallback(Ok(true))),
				Ok(0) => return Ok(ForkOut::Complete),
				Ok(n) => n,
				// No payload was copied; use ordinary DATA rather than activating an incomplete compressed file.
				Err(_) if written == 0 => return Ok(ForkOut::Fallback(Ok(true))),
				Err(e) => return Err(e),
			};

			match self.dist.write_at(c"com.apple.ResourceFork", &buf[..size], pos) {
				Ok(()) => {}
				Err(e) if written == 0 => return Ok(ForkOut::Fallback(Err(e))),
				Err(e) => return Err(e),
			}

			written += size as u64;
			copy.progress.update(written);
			copy.progress.report(&copy.tx);
		}
	}
}

// --- ForkOut
enum ForkOut {
	Complete,
	Fallback(io::Result<bool>),
}
