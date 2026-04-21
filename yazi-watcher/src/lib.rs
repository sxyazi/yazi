yazi_macro::mod_pub!(local remote);

yazi_macro::mod_flat!(backend proxy reporter watched watchee watcher);

pub static WATCHED: yazi_shim::cell::RoCell<parking_lot::RwLock<Watched>> =
	yazi_shim::cell::RoCell::new();
pub static WATCHER: yazi_shim::cell::RoCell<tokio::sync::Semaphore> =
	yazi_shim::cell::RoCell::new();

pub fn init() {
	WATCHED.with(<_>::default);
	WATCHER.init(tokio::sync::Semaphore::new(1));

	local::init();
}
