yazi_macro::mod_flat!(symlinker);

#[cfg(unix)]
yazi_macro::mod_flat!(unix);

#[cfg(windows)]
yazi_macro::mod_flat!(windows);
