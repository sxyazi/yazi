#[macro_export]
macro_rules! config_preset {
	($name:literal) => {{
		#[cfg(debug_assertions)]
		{
			std::borrow::Cow::from(
				std::fs::read_to_string(concat!(
					env!("CARGO_MANIFEST_DIR"),
					"/preset/",
					$name,
					"-default.toml"
				))
				.expect(concat!("Failed to read 'yazi-config/preset/", $name, "-default.toml'")),
			)
		}
		#[cfg(not(debug_assertions))]
		{
			std::borrow::Cow::from(include_str!(concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/preset/",
				$name,
				"-default.toml"
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

#[macro_export]
macro_rules! theme_preset {
	($name:literal) => {{
		#[cfg(debug_assertions)]
		{
			std::borrow::Cow::from(
				std::fs::read_to_string(concat!(
					env!("CARGO_MANIFEST_DIR"),
					"/preset/theme-",
					$name,
					".toml"
				))
				.expect(concat!("Failed to read 'yazi-config/preset/theme-", $name, ".toml'")),
			)
		}
		#[cfg(not(debug_assertions))]
		{
			std::borrow::Cow::from(include_str!(concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/preset/theme-",
				$name,
				".toml"
			)))
		}
	}};
}
