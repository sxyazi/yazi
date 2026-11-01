yazi_macro::mod_pub!(local r#virtual);

yazi_macro::mod_flat!(backend proxy refresher reporter watched watchee watcher);

pub(crate) static WATCHED: parking_lot::RwLock<Watched> = parking_lot::RwLock::new(Watched::new());
pub static WATCHER: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);
