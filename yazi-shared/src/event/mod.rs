yazi_macro::mod_flat!(cmd cow event);

pub static NEED_RENDER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
