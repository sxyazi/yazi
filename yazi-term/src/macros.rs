#[macro_export]
macro_rules! bail {
	() => {
		return Err($crate::ParseError::Invalid)
	};
}
