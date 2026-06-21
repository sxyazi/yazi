yazi_macro::mod_pub!(clear input);

yazi_macro::mod_flat!(clipboard renderable renderables scrollable step);

pub fn init() { CLIPBOARD.with(<_>::default); }
