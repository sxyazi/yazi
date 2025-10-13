yazi_macro::mod_pub!(local remote);

yazi_macro::mod_flat!(backend reporter watched watcher);

pub static WATCHED: yazi_shared::RoCell<parking_lot::RwLock<Watched>> = yazi_shared::RoCell::new();
pub static WATCHER: yazi_shared::RoCell<tokio::sync::Semaphore> = yazi_shared::RoCell::new();

pub fn init() {
	WATCHED.with(<_>::default);
	WATCHER.init(tokio::sync::Semaphore::new(1));

	local::init();
}
