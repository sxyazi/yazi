yazi_macro::mod_flat!(handle tty);

#[cfg(windows)]
yazi_macro::mod_flat!(windows);

pub static TTY: yazi_shared::RoCell<Tty> = yazi_shared::RoCell::new();

pub fn init() { TTY.with(<_>::default); }
