#[macro_export]
macro_rules! ok_or_not_found {
	($result:expr, $not_found:expr) => {
		match $result {
			Ok(v) => v,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => $not_found,
			Err(e) => Err(e)?,
		}
	};
	($result:expr) => {
		ok_or_not_found!($result, Default::default())
	};
}
