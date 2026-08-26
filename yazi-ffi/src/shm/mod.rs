yazi_macro::mod_flat!(common);

#[cfg(unix)]
yazi_macro::mod_flat!(unix);

#[cfg(windows)]
yazi_macro::mod_flat!(windows);

#[cfg(not(any(unix, windows)))]
yazi_macro::mod_flat!(unsupported);
