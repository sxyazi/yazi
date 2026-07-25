fn main() {
	cfg_aliases::cfg_aliases! {
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
