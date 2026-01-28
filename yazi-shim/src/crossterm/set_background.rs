pub struct SetBackground<'a>(pub &'a str);

impl crossterm::Command for SetBackground<'_> {
	fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
		if self.0.is_empty() { Ok(()) } else { write!(f, "\x1b]11;{}\x1b\\", self.0) }
	}

	#[cfg(windows)]
	fn execute_winapi(&self) -> std::io::Result<()> { Ok(()) }
}
