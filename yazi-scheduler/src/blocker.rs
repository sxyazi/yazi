use tokio::sync::Semaphore;
use yazi_shared::RoCell;

pub static BLOCKER: RoCell<Semaphore> = RoCell::new();

pub(super) fn init_blocker() { BLOCKER.init(Semaphore::new(1)) }
