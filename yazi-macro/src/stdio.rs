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
