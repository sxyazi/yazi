use tokio::task::LocalSet;

use crate::RoCell;

pub static LOCAL_SET: RoCell<LocalSet> = RoCell::new();
