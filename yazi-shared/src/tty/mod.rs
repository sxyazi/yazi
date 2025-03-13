yazi_macro::mod_flat!(handle tty);

#[cfg(windows)]
yazi_macro::mod_flat!(windows);

pub static TTY: crate::RoCell<Tty> = crate::RoCell::new();

pub(super) fn init() { TTY.with(<_>::default); }
