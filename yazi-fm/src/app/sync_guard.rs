use std::{io::Write, sync::atomic::{AtomicU8, Ordering}};

use ratatui_core::layout::Position;
use yazi_macro::writef;
use yazi_tty::{TTY, sequence::{BeginSyncUpdate, EndSyncUpdate, MoveTo, SetCursorStyle, ShowCursor}};

static DEPTH: AtomicU8 = AtomicU8::new(0);

pub(super) struct SyncGuard {
	finished: bool,
}

impl SyncGuard {
	pub(super) fn enter() -> Self {
		if DEPTH.fetch_add(1, Ordering::Relaxed) == 0 {
			write!(TTY.writer(), "{BeginSyncUpdate}").ok();
		}

		Self { finished: false }
	}

	pub(super) fn finish(mut self, cursor: Option<(Position, SetCursorStyle)>) {
		self.finished = true;
		if DEPTH.fetch_sub(1, Ordering::Relaxed) != 1 {
			return;
		}

		_ = if let Some((Position { x, y }, style)) = cursor {
			writef!(TTY.writer(), "{style}{}{ShowCursor}{EndSyncUpdate}", MoveTo(x, y))
		} else {
			writef!(TTY.writer(), "{EndSyncUpdate}")
		};
	}
}

impl Drop for SyncGuard {
	fn drop(&mut self) {
		if self.finished {
			return;
		}

		if DEPTH.fetch_sub(1, Ordering::Relaxed) == 1 {
			writef!(TTY.writer(), "{EndSyncUpdate}").ok();
		}
	}
}
