use std::sync::OnceLock;

use crate::auth::{Auth, AuthInventory, AuthKind};

pub fn init_tests() {
	static INIT: OnceLock<()> = OnceLock::new();

	INIT.get_or_init(crate::init);
}

inventory::submit! {
	AuthInventory {
		get: |scheme, domain| match (scheme.as_str(), domain.as_ref()) {
			("test-mount", "7z") => Some(Auth::new(AuthKind::Mount, scheme.clone(), "7z")),
			("test-hub", _) => Some(Auth::new(AuthKind::Hub, scheme.clone(), domain.clone())),
			("test-scope", "aws") => Some(Auth::new(AuthKind::Scope, scheme.clone(), "aws")),
			("sftp", "vps") => Some(Auth::new(AuthKind::Sftp, scheme.clone(), "vps")),
			_ => None,
		},
	}
}
