#![allow(clippy::option_map_unit_fn)]

yazi_macro::mod_pub!(errors event shell theme translit url);

yazi_macro::mod_flat!(chars condition debounce either env id layer natsort number os rand ro_cell sync_cell terminal throttle time);

pub fn init() {
	#[cfg(unix)]
	USERS_CACHE.with(<_>::default);

	event::Event::init();
}
