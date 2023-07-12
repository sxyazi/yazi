use once_cell::sync::Lazy;
use tokio::sync::Semaphore;

pub static BLOCKER: Lazy<Semaphore> = Lazy::new(|| Semaphore::new(1));
