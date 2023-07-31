use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};

#[inline]
pub fn tty_size() -> winsize {
	unsafe {
		let s: winsize = std::mem::zeroed();
		ioctl(STDOUT_FILENO, TIOCGWINSZ, &s);
		s
	}
}

#[inline]
pub fn tty_ratio() -> (f64, f64) {
	let s = tty_size();
	(f64::from(s.ws_xpixel) / f64::from(s.ws_col), f64::from(s.ws_ypixel) / f64::from(s.ws_row))
}
