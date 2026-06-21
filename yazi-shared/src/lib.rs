extern crate self as yazi_shared;

yazi_macro::mod_pub!(any_data data event id loc path pool scheme shell strand translit url);

yazi_macro::mod_flat!(bytes chars completion_token condition debounce env kebab_cased_string last_value layer localset natsort non_empty_string os predictor snake_cased_string source tests throttle time);

pub fn init() {
	LOCAL_SET.with(tokio::task::LocalSet::new);

	LOG_LEVEL.replace(<_>::from(std::env::var("YAZI_LOG").unwrap_or_default()));

	#[cfg(unix)]
	USERS_CACHE.with(<_>::default);

	pool::init();
	event::Event::init();
}
