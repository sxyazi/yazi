yazi_macro::mod_flat!(action action_cow actions cmd de de_owned event);

pub type Replier = tokio::sync::mpsc::UnboundedSender<anyhow::Result<crate::data::Data>>;

pub static NEED_RENDER: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
