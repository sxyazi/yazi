use crossterm::cursor::SetCursorStyle;

pub struct RestoreCursor {
	pub shape: u8,
	pub blink: bool,
}

impl crossterm::Command for RestoreCursor {
	fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
		let (shape, shape_blink) = match self.shape {
			u8::MAX => (0, None),
			n => (n.max(1).div_ceil(2), Some(n.max(1) & 1 == 1)),
		};

		let blink = shape_blink.unwrap_or(self.blink);
		Ok(match shape {
			2 if blink => SetCursorStyle::BlinkingUnderScore.write_ansi(f)?,
			2 if !blink => SetCursorStyle::SteadyUnderScore.write_ansi(f)?,
			3 if blink => SetCursorStyle::BlinkingBar.write_ansi(f)?,
			3 if !blink => SetCursorStyle::SteadyBar.write_ansi(f)?,
			_ => SetCursorStyle::DefaultUserShape.write_ansi(f)?,
		})
	}

	#[cfg(windows)]
	fn execute_winapi(&self) -> std::io::Result<()> { Ok(()) }
}
