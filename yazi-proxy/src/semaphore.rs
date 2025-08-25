use tokio::sync::Semaphore;
use yazi_shared::RoCell;

pub static HIDER: RoCell<Semaphore> = RoCell::new();

pub(super) fn init_semaphore() { HIDER.init(Semaphore::new(1)); }
