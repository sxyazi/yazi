use super::Actions;

impl Actions {
	pub(super) fn triple() -> String {
		format!(
			"{} ({}-{})",
			env!("VERGEN_RUSTC_HOST_TRIPLE"),
			std::env::consts::OS,
			std::env::consts::ARCH
		)
	}
}
