pub struct RestoreBackground;

impl crossterm::Command for RestoreBackground {
	fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
		write!(f, "\x1b]111\x1b\\")
	}

	#[cfg(windows)]
	fn execute_winapi(&self) -> std::io::Result<()> { Ok(()) }
}
