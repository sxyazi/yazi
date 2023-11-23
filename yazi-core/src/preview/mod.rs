mod preview;
mod provider;

pub use preview::*;
use provider::*;

pub static COLLISION: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
