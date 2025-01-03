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

#[macro_export]
macro_rules! win_either {
	($a:expr, $b:expr) => {{
		#[cfg(windows)]
		{
			$a
		}
		#[cfg(not(windows))]
		{
			$b
		}
	}};
}
