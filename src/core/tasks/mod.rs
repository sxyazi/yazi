mod file;
mod precache;
mod process;
mod scheduler;
mod tasks;

pub(crate) use file::*;
pub use precache::*;
pub(crate) use process::*;
pub use scheduler::*;
pub use tasks::*;

pub const TASKS_PADDING: u16 = 2;
pub const TASKS_PERCENT: u16 = 80;
