mod macros;

yazi_macro::mod_pub!(options);

yazi_macro::mod_flat!(app cmp confirm input mgr pick semaphore tab tasks);

pub fn init() { crate::init_semaphore(); }
