#[macro_export]
macro_rules! setup {
	() => {
		if ::std::env::args_os()
			.skip(1)
			.take_while(|arg| arg != "--")
			.any(|arg| arg == "-V" || arg == "--version")
		{
			let name = match env!("CARGO_PKG_NAME") {
				"yazi-cli" => "Ya",
				"yazi-fm" => "Yazi",
				s => s,
			};

			yazi_macro::outln!("{name}\n{}", $crate::version_full()).ok();
			::std::process::exit(0);
		}
	};
}
