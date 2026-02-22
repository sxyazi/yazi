use tokio::sync::Semaphore;
use yazi_shared::RoCell;

pub static YIELD_TO_SUBPROCESS: RoCell<Semaphore> = RoCell::new();
