#[inline]
pub(crate) fn check_for(for_: Option<&str>) -> bool {
	match for_.as_ref().map(|s| s.as_ref()) {
		Some("unix") if cfg!(unix) => true,
		Some(os) if os == std::env::consts::OS => true,
		Some(_) => false,
		None => true,
	}
}
