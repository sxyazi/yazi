use std::sync::OnceLock;

pub fn init_tests() {
	static INIT: OnceLock<()> = OnceLock::new();

	INIT.get_or_init(|| crate::init().unwrap());
}
