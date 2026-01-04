use std::ops::Deref;

use anyhow::{anyhow, bail};
use bitflags::bitflags;

use crate::cha::ChaType;

bitflags! {
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
	pub struct ChaMode: u16 {
		// File type
		const T_MASK   = 0b1111_0000_0000_0000;
		const T_SOCK   = 0b1100_0000_0000_0000;
		const T_LINK   = 0b1010_0000_0000_0000;
		const T_FILE   = 0b1000_0000_0000_0000;
		const T_BLOCK  = 0b0110_0000_0000_0000;
		const T_DIR    = 0b0100_0000_0000_0000;
		const T_CHAR   = 0b0010_0000_0000_0000;
		const T_FIFO   = 0b0001_0000_0000_0000;
		// Special
		const S_SUID   = 0b0000_1000_0000_0000;
		const S_SGID   = 0b0000_0100_0000_0000;
		const S_STICKY = 0b0000_0010_0000_0000;
		// User
		const U_MASK   = 0b0000_0001_1100_0000;
		const U_READ   = 0b0000_0001_0000_0000;
		const U_WRITE  = 0b0000_0000_1000_0000;
		const U_EXEC   = 0b0000_0000_0100_0000;
		// Group
		const G_MASK   = 0b0000_0000_0011_1000;
		const G_READ   = 0b0000_0000_0010_0000;
		const G_WRITE  = 0b0000_0000_0001_0000;
		const G_EXEC   = 0b0000_0000_0000_1000;
		// Others
		const O_MASK   = 0b0000_0000_0000_0111;
		const O_READ   = 0b0000_0000_0000_0100;
		const O_WRITE  = 0b0000_0000_0000_0010;
		const O_EXEC   = 0b0000_0000_0000_0001;
	}
}

impl Deref for ChaMode {
	type Target = ChaType;

	#[inline]
	fn deref(&self) -> &Self::Target {
		match *self & Self::T_MASK {
			Self::T_FILE => &ChaType::File,
			Self::T_DIR => &ChaType::Dir,
			Self::T_LINK => &ChaType::Link,
			Self::T_BLOCK => &ChaType::Block,
			Self::T_CHAR => &ChaType::Char,
			Self::T_SOCK => &ChaType::Sock,
			Self::T_FIFO => &ChaType::FIFO,
			_ => &ChaType::Unknown,
		}
	}
}

impl TryFrom<u16> for ChaMode {
	type Error = anyhow::Error;

	fn try_from(value: u16) -> Result<Self, Self::Error> {
		let me = Self::from_bits(value).ok_or_else(|| anyhow!("invalid file mode: {value:04o}"))?;
		match me & Self::T_MASK {
			Self::T_FILE
			| Self::T_DIR
			| Self::T_LINK
			| Self::T_BLOCK
			| Self::T_CHAR
			| Self::T_SOCK
			| Self::T_FIFO => Ok(me),
			_ => bail!("invalid file type: {value:04o}"),
		}
	}
}

#[cfg(unix)]
impl From<ChaMode> for std::fs::Permissions {
	fn from(value: ChaMode) -> Self {
		use std::os::unix::fs::PermissionsExt;

		Self::from_mode(value.bits() as _)
	}
}

impl ChaMode {
	// Convert a file mode to a string representation
	#[cfg(unix)]
	#[allow(clippy::collapsible_else_if)]
	pub fn permissions(self, dummy: bool) -> [u8; 10] {
		let mut s = *b"-?????????";

		// File type
		s[0] = match *self {
			ChaType::Dir => b'd',
			ChaType::Link => b'l',
			ChaType::Block => b'b',
			ChaType::Char => b'c',
			ChaType::Sock => b's',
			ChaType::FIFO => b'p',
			_ => b'-',
		};
		if dummy {
			return s;
		}

		// User
		s[1] = if self.contains(Self::U_READ) { b'r' } else { b'-' };
		s[2] = if self.contains(Self::U_WRITE) { b'w' } else { b'-' };
		s[3] = if self.contains(Self::U_EXEC) {
			if self.contains(Self::S_SUID) { b's' } else { b'x' }
		} else {
			if self.contains(Self::S_SUID) { b'S' } else { b'-' }
		};

		// Group
		s[4] = if self.contains(Self::G_READ) { b'r' } else { b'-' };
		s[5] = if self.contains(Self::G_WRITE) { b'w' } else { b'-' };
		s[6] = if self.contains(Self::G_EXEC) {
			if self.contains(Self::S_SGID) { b's' } else { b'x' }
		} else {
			if self.contains(Self::S_SGID) { b'S' } else { b'-' }
		};

		// Others
		s[7] = if self.contains(Self::O_READ) { b'r' } else { b'-' };
		s[8] = if self.contains(Self::O_WRITE) { b'w' } else { b'-' };
		s[9] = if self.contains(Self::O_EXEC) {
			if self.contains(Self::S_STICKY) { b't' } else { b'x' }
		} else {
			if self.contains(Self::S_STICKY) { b'T' } else { b'-' }
		};

		s
	}

	pub(super) fn from_bare(r#type: ChaType) -> Self {
		match r#type {
			ChaType::File => Self::T_FILE,
			ChaType::Dir => Self::T_DIR,
			ChaType::Link => Self::T_LINK,
			ChaType::Block => Self::T_BLOCK,
			ChaType::Char => Self::T_CHAR,
			ChaType::Sock => Self::T_SOCK,
			ChaType::FIFO => Self::T_FIFO,
			ChaType::Unknown => Self::empty(),
		}
	}
}

impl ChaMode {
	// TODO: deprecate
	#[inline]
	pub const fn is_exec(self) -> bool { self.contains(Self::U_EXEC) }

	#[inline]
	pub const fn is_sticky(self) -> bool { self.contains(Self::S_STICKY) }
}
