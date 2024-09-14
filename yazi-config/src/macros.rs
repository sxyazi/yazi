#[macro_export]
macro_rules! preset {
	($name:literal) => {{
		#[cfg(debug_assertions)]
		{
			std::borrow::Cow::Owned(
				std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/preset/", $name, ".toml"))
					.expect(concat!("Failed to read 'yazi-config/preset/", $name, ".toml'")),
			)
		}
		#[cfg(not(debug_assertions))]
		{
			std::borrow::Cow::Borrowed(include_str!(concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/preset/",
				$name,
				".toml"
			)))
		}
	}};
}
