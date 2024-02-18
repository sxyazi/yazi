use std::time::{Duration, Instant};

use unicode_width::UnicodeWidthStr;
use yazi_shared::event::Cmd;

use super::{Level, NOTIFY_BORDER};

pub struct Message {
	pub title:   String,
	pub content: String,
	pub level:   Level,

	pub instant: Instant,
	pub timeout: Duration,

	pub percent: u8,
}

impl TryFrom<Cmd> for Message {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		let timeout = c.take_name("timeout").and_then(|s| s.parse::<f64>().ok()).ok_or(())?;
		if timeout < 0.0 {
			return Err(());
		}

		let content = c.take_name("content").ok_or(())?;
		Ok(Self {
			title: c.take_name("title").ok_or(())?,
			content,
			level: c.take_name("level").ok_or(())?.parse()?,

			instant: Instant::now(),
			timeout: Duration::from_secs_f64(timeout),

			percent: 0,
		})
	}
}

impl Message {
	#[inline]
	pub fn height(&self, width: u16) -> usize {
		let lines = (self.content.width() as f64 / width as f64).ceil();
		lines as usize + NOTIFY_BORDER as usize
	}
}
