yazi_macro::mod_flat!(option semaphore state term);

pub fn init() { YIELD_TO_SUBPROCESS.init(tokio::sync::Semaphore::new(1)); }
