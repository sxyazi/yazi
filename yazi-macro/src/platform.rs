#[macro_export]
macro_rules! unix_either {
	($a:expr, $b:expr) => {{
		#[cfg(unix)]
		{
			$a
		}
		#[cfg(not(unix))]
		{
			$b
		}
	}};
}
