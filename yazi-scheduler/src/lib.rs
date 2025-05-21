#![allow(clippy::option_map_unit_fn, clippy::unit_arg)]

yazi_macro::mod_pub!(file plugin prework process);

yazi_macro::mod_flat!(ongoing out r#in scheduler task);

const LOW: u8 = yazi_config::Priority::Low as u8;
const NORMAL: u8 = yazi_config::Priority::Normal as u8;
const HIGH: u8 = yazi_config::Priority::High as u8;
