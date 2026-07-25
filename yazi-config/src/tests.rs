use std::sync::OnceLock;

use crate::{Preset, VFS};

pub fn init_tests() {
	static INIT: OnceLock<()> = OnceLock::new();

	INIT.get_or_init(|| VFS.init(Preset::vfs().unwrap()));
}
