use std::io;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Dimension {
	pub cols:   u16,
	pub rows:   u16,
	pub width:  u16,
	pub height: u16,
}

#[cfg(unix)]
impl From<rustix::termios::Winsize> for Dimension {
	fn from(size: rustix::termios::Winsize) -> Self {
		Self {
			cols:   size.ws_col,
			rows:   size.ws_row,
			width:  size.ws_xpixel,
			height: size.ws_ypixel,
		}
	}
}

impl Dimension {
	pub fn checked(self) -> io::Result<Self> {
		if self.cols == 0 || self.rows == 0 {
			Err(io::Error::other(
				"failed to get terminal dimension with both ioctl and $LINES/$COLUMNS variables",
			))
		} else {
			Ok(self)
		}
	}

	pub fn area(self) -> (u16, u16) { (self.cols, self.rows) }

	pub fn ratio(self) -> Option<(f64, f64)> {
		if self.rows == 0 || self.cols == 0 || self.width == 0 || self.height == 0 {
			None
		} else {
			Some((self.width as f64 / self.cols as f64, self.height as f64 / self.rows as f64))
		}
	}
}
