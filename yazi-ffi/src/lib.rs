yazi_macro::mod_pub!(device locale mount shm);

#[cfg(target_os = "macos")]
yazi_macro::mod_flat!(io_kit);

#[cfg(windows)]
yazi_macro::mod_flat!(com);
