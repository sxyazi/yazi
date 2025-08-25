yazi_macro::mod_pub!(backend);

yazi_macro::mod_flat!(linked watched watcher);

pub static LINKED: yazi_shared::RoCell<parking_lot::RwLock<Linked>> = yazi_shared::RoCell::new();
pub static WATCHED: yazi_shared::RoCell<parking_lot::RwLock<Watched>> = yazi_shared::RoCell::new();
pub static WATCHER: yazi_shared::RoCell<tokio::sync::Semaphore> = yazi_shared::RoCell::new();

pub fn init() {
	LINKED.with(<_>::default);
	WATCHED.with(<_>::default);
	WATCHER.init(tokio::sync::Semaphore::new(1));
}
