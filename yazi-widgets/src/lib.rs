yazi_macro::mod_pub!(input);

yazi_macro::mod_flat!(clear clipboard scrollable step);

pub fn init() { CLIPBOARD.with(<_>::default); }
