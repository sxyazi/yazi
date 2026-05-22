yazi_macro::mod_flat!(common waker);

#[cfg(unix)]
yazi_macro::mod_flat!(unix);

#[cfg(windows)]
yazi_macro::mod_flat!(windows);
