#![allow(clippy::option_map_unit_fn)]

yazi_macro::mod_pub!(data errors event loc path pool scheme shell strand translit url);

yazi_macro::mod_flat!(alias bytes chars condition debounce either env id layer natsort os predictor ro_cell source sync_cell terminal tests throttle time utf8 wtf8);

pub fn init() {
	pool::init();

	LOG_LEVEL.replace(<_>::from(std::env::var("YAZI_LOG").unwrap_or_default()));

	#[cfg(unix)]
	USERS_CACHE.with(<_>::default);

	event::Event::init();
}
