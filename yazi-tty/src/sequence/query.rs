use std::fmt::{self, Display};

use base64::{Engine, engine::general_purpose};
use yazi_ffi::shm::NamedSharedMemory;

/// XTVERSION request (secondary DA)
pub struct RequestXtVersion;

impl Display for RequestXtVersion {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[>q") }
}

/// Request character cell pixel size
pub struct RequestCellPixelSize;

impl Display for RequestCellPixelSize {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[16t") }
}

/// Request the current color scheme
pub struct RequestColorScheme;

impl Display for RequestColorScheme {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[?996n") }
}

/// Request background color via OSC 11
pub struct RequestBgColor;

impl Display for RequestBgColor {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b]11;?\x07") }
}

/// Request device attributes DA1
pub struct RequestDA1;

impl Display for RequestDA1 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[0c") }
}

/// Query Kitty graphics protocol capabilities
pub struct RequestKgp;

impl Display for RequestKgp {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("\x1b_Gi=278941603,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\")
	}
}

/// Query Kitty graphics protocol capabilities through shared memory.
pub struct RequestKgpShm {
	shm: Option<NamedSharedMemory>,
}

impl RequestKgpShm {
	pub fn new() -> Self { Self { shm: NamedSharedMemory::new(&[0; 3]).ok() } }
}

impl Display for RequestKgpShm {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if let Some(shm) = &self.shm {
			write!(
				f,
				"\x1b_Gi=916472805,s=1,v=1,a=q,t=s,f=24,S=3;{}\x1b\\",
				general_purpose::STANDARD.encode(&shm.name),
			)?;
		}

		Ok(())
	}
}

/// Request cursor style via DECRQSS (DECSCUSR)
pub struct RequestCursorStyle;

impl Display for RequestCursorStyle {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1bP$q q\x1b\\") }
}

/// Request cursor blink status via DECRQM (DECSET 12)
pub struct RequestCursorBlink;

impl Display for RequestCursorBlink {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[?12$p") }
}
