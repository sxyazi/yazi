yazi_macro::mod_pub!(options);

yazi_macro::mod_flat!(app, completion, confirm, input, manager, select, semaphore, tab, tasks);

pub fn init() { crate::init_semaphore(); }
