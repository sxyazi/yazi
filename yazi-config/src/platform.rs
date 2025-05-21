#[inline]
pub(crate) fn check_for(r#for: Option<&str>) -> bool {
	match r#for.as_ref().map(|s| s.as_ref()) {
		Some("unix") if cfg!(unix) => true,
		Some(os) if os == std::env::consts::OS => true,
		Some(_) => false,
		None => true,
	}
}
