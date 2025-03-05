#[macro_export]
macro_rules! time {
	($expr:expr) => {
		$crate::time!(stringify!($expr), $expr)
	};
	($label:expr, $expr:expr) => {
		$crate::time!($expr, "{}", $label)
	};
	($expr:expr, $fmt:expr, $($args:tt)*) => {{
		if tracing::enabled!(tracing::Level::DEBUG) {
			let start = std::time::Instant::now();
			let result = $expr;
			tracing::debug!("{} took {:?}", format_args!($fmt, $($args)*), start.elapsed());
			result
		} else {
			$expr
		}
	}};
}
