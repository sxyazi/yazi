yazi_macro::mod_flat!(tty handle);

pub static TTY: crate::RoCell<Tty> = crate::RoCell::new();

pub(super) fn init() { TTY.with(<_>::default); }
