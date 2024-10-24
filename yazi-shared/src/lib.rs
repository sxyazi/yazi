#![allow(clippy::option_map_unit_fn)]

yazi_macro::mod_pub!(errors event fs shell theme translit);

yazi_macro::mod_flat!(chars condition debounce env id layer natsort number os rand ro_cell terminal throttle time xdg);

pub fn init() {
	#[cfg(unix)]
	USERS_CACHE.with(<_>::default);

	event::Event::init();
}
