#[macro_export]
macro_rules! outln {
	($($tt:tt)*) => {{
		use std::io::Write;
		writeln!(std::io::stdout(), $($tt)*)
	}}
}

#[macro_export]
macro_rules! errln {
	($($tt:tt)*) => {{
		use std::io::Write;
		writeln!(std::io::stderr(), $($tt)*)
	}}
}

/// Like [`write!`] but immediately flushes the writer afterwards.
#[macro_export]
macro_rules! writef {
	($dst:expr, $($arg:tt)*) => {{
		use std::io::Write as _;
		write!($dst, $($arg)*).and_then(|_| ($dst).flush())
	}};
}
