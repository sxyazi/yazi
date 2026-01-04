yazi_macro::mod_pub!(tty);

yazi_macro::mod_flat!(background cursor r#if);

pub fn init() { tty::init(); }
