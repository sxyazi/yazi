mod macros;

yazi_macro::mod_pub!(custom fetch file hook plugin preload process size);

yazi_macro::mod_flat!(behavior cleanup handle loaded ongoing op out progress proxy r#in scheduler snap status summary task worker);

const LOW: u8 = yazi_config::Priority::Low as u8;
const NORMAL: u8 = yazi_config::Priority::Normal as u8;
const HIGH: u8 = yazi_config::Priority::High as u8;
