yazi_macro::mod_flat!(handle lua reader tty writer);

yazi_macro::mod_pub!(sequence);

pub static TTY: yazi_shim::cell::RoCell<Tty> = yazi_shim::cell::RoCell::new();

pub fn init() { TTY.with(<_>::default); }
