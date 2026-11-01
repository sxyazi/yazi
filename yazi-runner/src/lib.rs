yazi_macro::mod_pub!(entry evaluator fetcher loader preloader previewer provider spot sync);

yazi_macro::mod_flat!(coroutine inventory runner traits);

pub fn init() { crate::loader::init(); }
