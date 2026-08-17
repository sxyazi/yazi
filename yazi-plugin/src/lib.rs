yazi_macro::mod_pub!(external fs keymap pubsub runtime tasks theme ui utils);

yazi_macro::mod_flat!(slim standard);

pub(crate) static HTTP: yazi_shim::cell::RoCell<reqwest::Client> = yazi_shim::cell::RoCell::new();

pub fn setup() -> anyhow::Result<()> {
	HTTP.init(
		reqwest::Client::builder()
			.user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36")
			.build()?,
	);

	LUA.init(crate::standard_lua()?);

	Ok(())
}
