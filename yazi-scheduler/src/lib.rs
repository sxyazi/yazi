mod macros;

yazi_macro::mod_pub!(file hook plugin prework process);

yazi_macro::mod_flat!(ongoing op out progress r#in runner scheduler snap task);

const LOW: u8 = yazi_config::Priority::Low as u8;
const NORMAL: u8 = yazi_config::Priority::Normal as u8;
const HIGH: u8 = yazi_config::Priority::High as u8;
