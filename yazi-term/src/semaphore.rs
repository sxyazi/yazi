use tokio::sync::Semaphore;

pub static YIELD_TO_SUBPROCESS: Semaphore = Semaphore::const_new(1);
