#![allow(clippy::option_map_unit_fn, clippy::unit_arg)]

mod blocker;
mod file;
mod op;
mod plugin;
mod preload;
mod process;
mod running;
mod scheduler;
mod task;

pub use blocker::*;
pub use op::*;
pub use running::*;
pub use scheduler::*;
pub use task::*;

const LOW: u8 = yazi_config::Priority::Low as u8;
const NORMAL: u8 = yazi_config::Priority::Normal as u8;
const HIGH: u8 = yazi_config::Priority::High as u8;

pub fn init() { init_blocker(); }
