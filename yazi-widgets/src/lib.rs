yazi_macro::mod_pub!(input);

yazi_macro::mod_flat!(clipboard scrollable);

pub fn init() { CLIPBOARD.with(<_>::default); }
