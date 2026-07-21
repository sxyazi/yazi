yazi_macro::mod_flat!(entry lua node nodes);

#[cfg(target_os = "macos")]
yazi_macro::mod_flat!(macos);

#[cfg(windows)]
yazi_macro::mod_flat!(windows);

#[cfg(all(unix, not(target_os = "macos"), not(target_os = "android"), not(target_os = "ios")))]
yazi_macro::mod_flat!(freedesktop);

#[cfg(any(target_os = "android", target_os = "ios"))]
yazi_macro::mod_flat!(unsupported);

#[cfg(not(any(target_os = "android", target_os = "ios")))]
yazi_macro::mod_flat!(traits);
