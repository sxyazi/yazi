pub struct SetBackground(pub bool, pub String);

impl crossterm::Command for SetBackground {
	fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
		if self.1.is_empty() {
			Ok(())
		} else if self.0 {
			write!(f, "\x1b]11;{}\x1b\\", self.1)
		} else {
			write!(f, "\x1b]111\x1b\\")
		}
	}

	#[cfg(windows)]
	fn execute_winapi(&self) -> std::io::Result<()> { Ok(()) }
}
