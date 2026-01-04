yazi_macro::mod_pub!(data errors event loc path pool scheme shell strand translit url wtf8);

yazi_macro::mod_flat!(alias bytes chars completion_token condition debounce either env id layer localset natsort os predictor ro_cell source sync_cell terminal tests throttle time utf8);

pub fn init() {
	LOCAL_SET.with(tokio::task::LocalSet::new);

	LOG_LEVEL.replace(<_>::from(std::env::var("YAZI_LOG").unwrap_or_default()));

	#[cfg(unix)]
	USERS_CACHE.with(<_>::default);

	pool::init();
	event::Event::init();
}
