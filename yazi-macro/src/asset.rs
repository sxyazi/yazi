#[macro_export]
macro_rules! config_preset {
	($name:literal) => {{
		#[cfg(debug_assertions)]
		{
			std::borrow::Cow::from(
				std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/preset/", $name, ".toml"))
					.expect(concat!("Failed to read 'yazi-config/preset/", $name, ".toml'")),
			)
		}
		#[cfg(not(debug_assertions))]
		{
			std::borrow::Cow::from(include_str!(concat!(
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

#[macro_export]
macro_rules! theme_preset {
	($name:literal) => {{
		#[cfg(debug_assertions)]
		{
			let append = std::fs::read_to_string(concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/preset/theme+",
				$name,
				".toml"
			))
			.expect(concat!("Failed to read 'yazi-config/preset/theme+", $name, ".toml'"));
			std::borrow::Cow::from(format!("{}\n{append}", &$crate::config_preset!("theme")))
		}
		#[cfg(not(debug_assertions))]
		{
			std::borrow::Cow::from(concat!(
				include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/preset/theme.toml")),
				"\n",
				include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/preset/theme+", $name, ".toml"))
			))
		}
	}};
}
