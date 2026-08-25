yazi_macro::mod_pub!(local r#virtual);

yazi_macro::mod_flat!(backend proxy refresher reporter watched watchee watcher);

pub(crate) static WATCHED: yazi_shim::cell::RoCell<parking_lot::RwLock<Watched>> =
	yazi_shim::cell::RoCell::new();
pub static WATCHER: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

pub fn init() {
	WATCHED.with(<_>::default);

	local::init();
}
