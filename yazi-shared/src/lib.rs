#![allow(clippy::option_map_unit_fn)]

yazi_macro::mod_pub!(errors event shell theme translit url);

yazi_macro::mod_flat!(chars condition debounce either env id layer natsort number os rand ro_cell sync_cell terminal throttle time utf8);

pub fn init() {
	LOG_LEVEL.replace(<_>::from(std::env::var("YAZI_LOG").unwrap_or_default()));

	#[cfg(unix)]
	USERS_CACHE.with(<_>::default);

	event::Event::init();
}
