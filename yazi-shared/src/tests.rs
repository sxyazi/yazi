pub fn init_tests() {
	static INIT: std::sync::OnceLock<()> = std::sync::OnceLock::new();

	INIT.get_or_init(|| {
		crate::init();
	});
}
