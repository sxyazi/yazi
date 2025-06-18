#[macro_export]
macro_rules! try_format {
	($($arg:tt)*) => {{
		use std::fmt::Write;

		fn inner(args: std::fmt::Arguments<'_>) -> Result<String, std::fmt::Error> {
			if let Some(s) = args.as_str() {
				Ok(s.to_owned())
			} else {
				let mut output = String::new();
				output.write_fmt(args).map(|_| output)
			}
		}

		inner(format_args!($($arg)*))
	}}
}
