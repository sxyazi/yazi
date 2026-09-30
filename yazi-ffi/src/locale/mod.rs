yazi_macro::mod_flat!(error locale);

#[cfg(any(windows, all(unix, not(target_os = "macos"))))]
yazi_macro::mod_flat!(common);

#[cfg(target_os = "macos")]
yazi_macro::mod_flat!(macos);

#[cfg(all(unix, not(target_os = "macos")))]
yazi_macro::mod_flat!(unix);

#[cfg(windows)]
yazi_macro::mod_flat!(windows);
