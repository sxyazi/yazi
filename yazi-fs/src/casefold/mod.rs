yazi_macro::mod_flat!(common);

#[cfg(any(target_os = "linux", target_os = "android"))]
yazi_macro::mod_flat!(linux);

#[cfg(target_os = "macos")]
yazi_macro::mod_flat!(macos);

#[cfg(windows)]
yazi_macro::mod_flat!(windows);

#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
yazi_macro::mod_flat!(bsd);

#[cfg(any(
	target_os = "linux",
	target_os = "android",
	target_os = "freebsd",
	target_os = "netbsd",
	target_os = "openbsd"
))]
yazi_macro::mod_flat!(unix);
