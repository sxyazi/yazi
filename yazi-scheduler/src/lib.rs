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

const VERY_LOW: u8 = 0;
const LOW: u8 = 1;
const NORMAL: u8 = 2;
const HIGH: u8 = 3;
const VERY_HIGH: u8 = 4;

pub fn init() { init_blocker(); }
