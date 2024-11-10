use super::Actions;

impl Actions {
	pub(super) fn rustc() -> String {
		format!(
			"{} ({} {})",
			env!("VERGEN_RUSTC_SEMVER"),
			&env!("VERGEN_RUSTC_COMMIT_HASH")[..8],
			env!("VERGEN_RUSTC_COMMIT_DATE")
		)
	}
}
