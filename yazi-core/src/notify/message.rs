use std::time::{Duration, Instant};

use unicode_width::UnicodeWidthStr;
use yazi_proxy::options::{NotifyLevel, NotifyOpt};

use super::NOTIFY_BORDER;

pub struct Message {
	pub title:   String,
	pub content: String,
	pub level:   NotifyLevel,
	pub timeout: Duration,

	pub instant: Instant,
	pub percent: u8,
}

impl From<NotifyOpt> for Message {
	fn from(opt: NotifyOpt) -> Self {
		Self {
			title:   opt.title,
			content: opt.content,
			level:   opt.level,
			timeout: opt.timeout,

			instant: Instant::now(),
			percent: 0,
		}
	}
}

impl Message {
	#[inline]
	pub fn height(&self, width: u16) -> usize {
		if width == 0 {
			return 0; // In case we can't get the width of the terminal
		}

		let mut lines = 0;
		for line in self.content.lines() {
			lines += (line.width() + 1).div_ceil(width as usize)
		}

		lines + NOTIFY_BORDER as usize
	}
}
