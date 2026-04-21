use tokio::task::LocalSet;
use yazi_shim::cell::RoCell;

pub static LOCAL_SET: RoCell<LocalSet> = RoCell::new();
