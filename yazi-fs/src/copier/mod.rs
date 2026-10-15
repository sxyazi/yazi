yazi_macro::mod_flat!(copier progress);

#[cfg(unix)]
yazi_macro::mod_flat!(common);

#[cfg(any(target_os = "linux", target_os = "android"))]
yazi_macro::mod_flat!(linux);

#[cfg(target_os = "macos")]
yazi_macro::mod_flat!(copystate decmpfs macos);

#[cfg(windows)]
yazi_macro::mod_flat!(windows);

#[cfg(all(unix, not(any(target_os = "linux", target_os = "android", target_os = "macos"))))]
yazi_macro::mod_flat!(fallback);
