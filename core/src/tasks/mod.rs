mod running;
mod scheduler;
mod tasks;
mod workers;

use running::*;
use scheduler::*;
pub use tasks::*;

pub const TASKS_PADDING: u16 = 2;
pub const TASKS_PERCENT: u16 = 80;
