use shared::RoCell;
use tokio::sync::Semaphore;

pub static BLOCKER: RoCell<Semaphore> = RoCell::new();

pub(super) fn init_blocker() { BLOCKER.init(Semaphore::new(1)) }
