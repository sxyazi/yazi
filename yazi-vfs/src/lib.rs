#[macro_use]
mod macros;

yazi_macro::mod_pub!(engine remover);

yazi_macro::mod_flat!(stat entries file fns http stamp);

pub fn init() { engine::init(); }
