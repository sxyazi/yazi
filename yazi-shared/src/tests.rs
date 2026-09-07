use std::sync::OnceLock;

use crate::auth::{AuthArc, AuthInventory, AuthKind};

pub fn init_tests() {
	static INIT: OnceLock<()> = OnceLock::new();

	INIT.get_or_init(crate::init);
}

inventory::submit! {
	AuthInventory {
		get: |scheme, domain| match (scheme.as_str(), domain.as_ref()) {
			("test-mount", b"7z") => Some(AuthArc::new(AuthKind::Mount, scheme.clone(), "7z")),
			("test-hub", _) => Some(AuthArc::new(AuthKind::Hub, scheme.clone(), domain.clone())),
			("test-scope", b"aws") => Some(AuthArc::new(AuthKind::Scope, scheme.clone(), "aws")),
			("test-view", b"fx") => Some(AuthArc::new(AuthKind::View, scheme.clone(), "fx")),
			("sftp", b"vps") => Some(AuthArc::new(AuthKind::Sftp, scheme.clone(), "vps")),
			_ => None,
		},
	}
}
