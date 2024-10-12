#[macro_export]
macro_rules! config_preset {
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

#[macro_export]
macro_rules! plugin_preset {
	($name:literal) => {{
		#[cfg(debug_assertions)]
		{
			std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/preset/", $name, ".lua")).expect(concat!(
				"Failed to read 'yazi-plugin/preset/",
				$name,
				".lua'"
			))
		}
		#[cfg(not(debug_assertions))]
		{
			&include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/preset/", $name, ".lua"))[..]
		}
	}};
}
