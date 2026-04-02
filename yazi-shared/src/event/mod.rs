yazi_macro::mod_flat!(action cow de de_owned event);

pub static NEED_RENDER: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
