fn main() {
	println!("cargo:rerun-if-changed=build.rs");

	aliases();
}

fn aliases() {
	cfg_aliases::cfg_aliases! {
		// Mounts
		mount_unsupported: {
			not(any(target_os = "linux", target_os = "macos", windows, mount_bsd))
		},
		mount_bsd: {
			any(target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd", target_os = "netbsd")
		},

		// Trash
		trash_unsupported: {
			any(target_os = "android", target_os = "ios")
		},
		trash_unix: {
			all(unix, not(trash_unsupported))
		},
		trash_freedesktop: {
			all(unix, not(target_os = "macos"), not(trash_unsupported))
		},
	}
}
