yazi_macro::mod_flat!(action action_cow actions channel cmd de de_owned event replier);

pub static NEED_RENDER: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
