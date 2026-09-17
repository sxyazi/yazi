#[cfg(any(
	target_os = "linux",
	target_os = "android",
	target_os = "freebsd",
	target_os = "netbsd",
	target_os = "openbsd"
))]
yazi_macro::mod_flat!(scanner);
