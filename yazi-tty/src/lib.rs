yazi_macro::mod_flat!(handle tty);

#[cfg(windows)]
yazi_macro::mod_flat!(windows);

pub static TTY: yazi_shim::cell::RoCell<Tty> = yazi_shim::cell::RoCell::new();

pub fn init() { TTY.with(<_>::default); }
