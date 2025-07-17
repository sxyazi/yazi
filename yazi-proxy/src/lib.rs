mod macros;

yazi_macro::mod_flat!(app cmp confirm input mgr pick semaphore tasks which);

pub fn init() { crate::init_semaphore(); }
