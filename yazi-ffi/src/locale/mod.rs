yazi_macro::mod_flat!(date_field date_parser error locale state strftime time_field time_parser width);

#[cfg(target_os = "macos")]
yazi_macro::mod_flat!(macos);

#[cfg(all(unix, not(target_os = "macos")))]
yazi_macro::mod_flat!(unix);

#[cfg(windows)]
yazi_macro::mod_flat!(windows);
