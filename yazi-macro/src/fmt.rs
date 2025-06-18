#[macro_export]
macro_rules! try_format {
	($($arg:tt)*) => {{
		use std::fmt::Write;

		let mut output = String::new();
		output.write_fmt(format_args!($($arg)*)).map(|_| output)
	}}
}
