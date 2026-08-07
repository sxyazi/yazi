extern crate self as yazi_shared;

yazi_macro::mod_pub!(any_data auth data spec event id loc path pool shell strand translit url);

yazi_macro::mod_flat!(bytes chars completion_cell completion_token condition debounce env kebab_cased_key last_value layer localset natsort non_empty_string os predictor snake_cased_key source tests throttle time);

pub fn init() {
	LOCAL_SET.with(tokio::task::LocalSet::new);

	#[cfg(unix)]
	USERS_CACHE.with(<_>::default);

	pool::init();
	auth::init();
	event::Event::init();
}
