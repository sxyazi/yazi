yazi_macro::mod_flat!(handle reader tty writer);

pub static TTY: yazi_shim::cell::RoCell<Tty> = yazi_shim::cell::RoCell::new();

pub fn init() { TTY.with(<_>::default); }
