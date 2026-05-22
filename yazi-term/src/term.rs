use yazi_shim::cell::RoCell;

use crate::terminal::Terminal;

pub static TERM: RoCell<Terminal<'static>> = RoCell::new();
