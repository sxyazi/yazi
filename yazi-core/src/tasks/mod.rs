mod commands;
mod file;
mod plugin;
mod preload;
mod process;
mod progress;
mod tasks;

pub use progress::*;
pub use tasks::*;

pub const TASKS_BORDER: u16 = 2;
pub const TASKS_PADDING: u16 = 2;
pub const TASKS_PERCENT: u16 = 80;
