pub struct If<T: crossterm::Command>(pub bool, pub T);

impl<T: crossterm::Command> crossterm::Command for If<T> {
	fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
		if self.0 { self.1.write_ansi(f) } else { Ok(()) }
	}

	#[cfg(windows)]
	fn execute_winapi(&self) -> std::io::Result<()> {
		if self.0 { self.1.execute_winapi() } else { Ok(()) }
	}

	#[cfg(windows)]
	fn is_ansi_code_supported(&self) -> bool { self.1.is_ansi_code_supported() }
}
