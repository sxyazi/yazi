use std::{fmt::{self, Display}, str};

use base64::{Engine, engine::general_purpose};

/// Enable drag support: `OSC 72 ; t=o:x=1 ; machine id ST`
pub struct EnableDrag<'a>(pub &'a str);

impl Display for EnableDrag<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b]72;t=o:x=1;{}\x1b\\", self.0)
	}
}

/// Enable drop support: `OSC 72 ; t=a ; MIME list ST`
pub struct EnableDrop<'a>(pub &'a [&'a str]);

impl Display for EnableDrop<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b]72;t=a;{}\x1b\\", ListDndMimes(self.0))
	}
}

/// Disable drag support: `OSC 72 ; t=o:x=2 ST`
pub struct DisableDrag;

impl Display for DisableDrag {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b]72;t=o:x=2\x1b\\") }
}

/// Disable drop support: `OSC 72 ; t=A ST`
pub struct DisableDrop;

impl Display for DisableDrop {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b]72;t=A\x1b\\") }
}

/// Confirm drag: `OSC 72 ; t=o:o=operation ST`
pub enum ConfirmDrag<'a> {
	Copy(&'a [&'a str]),
	Move(&'a [&'a str]),
	Either(&'a [&'a str]),
}

impl Display for ConfirmDrag<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Copy(mimes) => write!(f, "\x1b]72;t=o:o=1;{}\x1b\\", ListDndMimes(mimes)),
			Self::Move(mimes) => write!(f, "\x1b]72;t=o:o=2;{}\x1b\\", ListDndMimes(mimes)),
			Self::Either(mimes) => write!(f, "\x1b]72;t=o:o=3;{}\x1b\\", ListDndMimes(mimes)),
		}
	}
}

/// Confirm dropped data: `OSC 72 ; t=m:o=O ; MIME list ST`
pub enum ConfirmDrop<'a> {
	Reject,
	Copy(&'a [&'a str]),
	Move(&'a [&'a str]),
}

impl Display for ConfirmDrop<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Reject => write!(f, "\x1b]72;t=m:o=0\x1b\\"),
			Self::Copy(mimes) => write!(f, "\x1b]72;t=m:o=1;{}\x1b\\", ListDndMimes(mimes)),
			Self::Move(mimes) => write!(f, "\x1b]72;t=m:o=2;{}\x1b\\", ListDndMimes(mimes)),
		}
	}
}

/// Start dragging: `OSC 72 ; t=P:x=-1 ST`
pub struct StartDrag;

impl Display for StartDrag {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b]72;t=P:x=-1\x1b\\") }
}

/// Start requesting dropped data: `OSC 72 ; t=r:x=idx ST`
pub struct StartDrop(pub u8);

impl Display for StartDrop {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b]72;t=r:x={}\x1b\\", self.0)
	}
}

/// Present drag data: `OSC 72 ; t=p:x=idx ; base64 encoded data ST`
pub struct PresentDrag<'a>(pub u8, pub &'a [u8]);

impl Display for PresentDrag<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let b64 = general_purpose::STANDARD_NO_PAD.encode(self.1).into_bytes();
		let chunks = b64.len().div_ceil(4096);

		for (i, chunk) in b64.chunks(4096).enumerate() {
			let s = unsafe { str::from_utf8_unchecked(chunk) };
			if i == 0 {
				write!(f, "\x1b]72;t=p:x={}:m={};{s}\x1b\\", self.0, (chunks > 1) as u8)?;
			} else {
				write!(f, "\x1b]72;m={};{s}\x1b\\", (i + 1 < chunks) as u8)?;
			}
		}

		write!(f, "\x1b]72;t=p:x={}\x1b\\", self.0)
	}
}

/// Present drag icon data:
/// `OSC 72 ; t=p:x=-1:y=fmt:X=width:Y=height:o=opacity ; base64 payload ST`
pub struct PresentDragIcon<'a> {
	pub format:  u8,
	pub opacity: u16,
	pub width:   u32,
	pub height:  u32,
	pub payload: &'a [u8],
}

impl Display for PresentDragIcon<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let b64 = general_purpose::STANDARD_NO_PAD.encode(self.payload).into_bytes();
		let chunks = b64.len().div_ceil(4096);

		for (i, chunk) in b64.chunks(4096).enumerate() {
			let s = unsafe { str::from_utf8_unchecked(chunk) };
			if i == 0 {
				write!(
					f,
					"\x1b]72;t=p:x=-1:y={}:X={}:Y={}:o={}:m={};{s}\x1b\\",
					self.format,
					self.width,
					self.height,
					self.opacity,
					(chunks > 1) as u8
				)?;
			} else {
				write!(f, "\x1b]72;m={};{s}\x1b\\", (i + 1 < chunks) as u8)?;
			}
		}

		Ok(())
	}
}

/// Finish requesting dropped data: `OSC 72 ; t=r:o=operation ST`
#[derive(Clone, Copy)]
pub enum FinishDrop {
	Copy = 1,
	Move = 2,
}

impl Display for FinishDrop {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b]72;t=r:o={}\x1b\\", *self as u8)
	}
}

/// Write MIME types separated by spaces.
struct ListDndMimes<'a>(&'a [&'a str]);

impl Display for ListDndMimes<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for (i, &m) in self.0.iter().enumerate() {
			if i != 0 {
				write!(f, " ")?;
			}
			write!(f, "{m}")?;
		}
		Ok(())
	}
}
