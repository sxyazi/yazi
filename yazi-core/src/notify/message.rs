use std::time::{Duration, Instant};

use unicode_width::UnicodeWidthStr;
use yazi_proxy::options::{NotifyLevel, NotifyOpt};

use super::NOTIFY_BORDER;

pub struct Message {
	pub title:   String,
	pub content: String,
	pub level:   NotifyLevel,
	pub timeout: Duration,

	pub instant:   Instant,
	pub percent:   u8,
	pub max_width: usize,
}

impl From<NotifyOpt> for Message {
	fn from(opt: NotifyOpt) -> Self {
		let title = opt.title.lines().next().unwrap_or_default();
		let title_width = title.width() + (opt.level.icon().width() + /* Space */ 1);

		let max_width = opt.content.lines().map(|s| s.width()).max().unwrap_or(0).max(title_width);

		Self {
			title:   title.to_owned(),
			content: opt.content,
			level:   opt.level,
			timeout: opt.timeout,

			instant:   Instant::now(),
			percent:   0,
			max_width: max_width + NOTIFY_BORDER as usize,
		}
	}
}

impl Message {
	#[inline]
	pub fn height(&self, width: u16) -> usize {
		let lines = ratatui::widgets::Paragraph::new(self.content.as_str())
			.wrap(ratatui::widgets::Wrap { trim: false })
			.line_count(width);

		lines + NOTIFY_BORDER as usize
	}
}

impl PartialEq for Message {
	fn eq(&self, other: &Self) -> bool {
		self.level == other.level && self.content == other.content && self.title == other.title
	}
}
