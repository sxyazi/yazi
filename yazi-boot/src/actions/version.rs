use super::Actions;

impl Actions {
	pub fn version() -> &'static str {
		concat!(
			env!("CARGO_PKG_VERSION"),
			" (",
			env!("VERGEN_GIT_SHA"),
			" ",
			env!("VERGEN_BUILD_DATE"),
			")"
		)
	}
}
