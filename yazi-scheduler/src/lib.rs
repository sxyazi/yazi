#![allow(clippy::option_map_unit_fn, clippy::unit_arg)]

mod file;
mod ongoing;
mod op;
mod plugin;
mod preload;
mod process;
mod scheduler;
mod semaphore;
mod task;

pub use ongoing::*;
pub use op::*;
pub use scheduler::*;
pub use semaphore::*;
pub use task::*;

const LOW: u8 = yazi_config::Priority::Low as u8;
const NORMAL: u8 = yazi_config::Priority::Normal as u8;
const HIGH: u8 = yazi_config::Priority::High as u8;

pub fn init() { init_semaphore(); }
