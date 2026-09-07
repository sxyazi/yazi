extern crate self as yazi_shared;

yazi_macro::mod_pub!(any_data auth data domain event id loc path pool sendable shell spec strand translit twiddle url wire);

yazi_macro::mod_flat!(bytes chars completion_token condition debounce env kebab_cased_key last_value layer localset natsort non_empty_string os predictor snake_cased_key source tests throttle time);

pub fn init() {
	LOCAL_SET.with(tokio::task::LocalSet::new);

	#[cfg(unix)]
	USERS_CACHE.with(<_>::default);

	pool::init();
	event::Event::init();
}
