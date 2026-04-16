use std::time::{Duration, Instant};

use unicode_width::UnicodeWidthStr;

use super::NOTIFY_BORDER;
use crate::notify::{MessageLevel, MessageOpt};

pub struct Message {
	pub title:   String,
	pub content: String,
	pub level:   MessageLevel,
	pub timeout: Duration,

	pub instant: Instant,
	pub percent: u8,

	title_width:   usize, // Width of title without icon
	content_width: usize, // Width of longest line in content
}

impl From<MessageOpt> for Message {
	fn from(opt: MessageOpt) -> Self {
		let title = opt.title.lines().next().unwrap_or_default();
		let content_width = opt.content.lines().map(|s| s.width()).max().unwrap_or(0);

		Self {
			title: title.to_owned(),
			content: opt.content,
			level: opt.level,
			timeout: opt.timeout,

			instant: Instant::now(),
			percent: 0,

			title_width: title.width(),
			content_width,
		}
	}
}

impl PartialEq for Message {
	fn eq(&self, other: &Self) -> bool {
		self.level == other.level && self.content == other.content && self.title == other.title
	}
}

impl Message {
	pub fn width(&self) -> usize {
		let icon_width = self.level.icon().width() + /* Space */ 1;

		self.content_width.max(self.title_width + icon_width) + NOTIFY_BORDER as usize
	}

	pub fn height(&self, width: u16) -> usize {
		let lines = ratatui::widgets::Paragraph::new(self.content.as_str())
			.wrap(ratatui::widgets::Wrap { trim: false })
			.line_count(width.saturating_sub(NOTIFY_BORDER));

		lines + NOTIFY_BORDER as usize
	}
}
