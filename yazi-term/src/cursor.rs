use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use crossterm::cursor::SetCursorStyle;

static SHAPE: AtomicU8 = AtomicU8::new(0);
static BLINK: AtomicBool = AtomicBool::new(false);

pub struct RestoreCursor;

impl RestoreCursor {
	pub fn store(resp: &str) {
		SHAPE.store(
			resp
				.split_once("\x1bP1$r")
				.and_then(|(_, s)| s.bytes().next())
				.filter(|&b| matches!(b, b'0'..=b'6'))
				.map_or(u8::MAX, |b| b - b'0'),
			Ordering::Relaxed,
		);
		BLINK.store(resp.contains("\x1b[?12;1$y"), Ordering::Relaxed);
	}
}

impl crossterm::Command for RestoreCursor {
	fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
		let (shape, shape_blink) = match SHAPE.load(Ordering::Relaxed) {
			u8::MAX => (0, None),
			n => (n.max(1).div_ceil(2), Some(n.max(1) & 1 == 1)),
		};

		let blink = shape_blink.unwrap_or(BLINK.load(Ordering::Relaxed));
		Ok(match shape {
			2 if blink => SetCursorStyle::BlinkingUnderScore.write_ansi(f)?,
			2 if !blink => SetCursorStyle::SteadyUnderScore.write_ansi(f)?,
			3 if blink => SetCursorStyle::BlinkingBar.write_ansi(f)?,
			3 if !blink => SetCursorStyle::SteadyBar.write_ansi(f)?,
			_ if blink => SetCursorStyle::DefaultUserShape.write_ansi(f)?,
			_ if !blink => SetCursorStyle::SteadyBlock.write_ansi(f)?,
			_ => unreachable!(),
		})
	}

	#[cfg(windows)]
	fn execute_winapi(&self) -> std::io::Result<()> { Ok(()) }
}
