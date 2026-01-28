yazi_macro::mod_flat!(cmd cow event);

pub static NEED_RENDER: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
