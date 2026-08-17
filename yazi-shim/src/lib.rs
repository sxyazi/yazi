yazi_macro::mod_pub!(arc_swap cell fs log mlua path ratatui rustls serde strum toml vec wtf8);

yazi_macro::mod_flat!(option percent_encoding result sstr twox utf8);

#[cfg(windows)]
yazi_macro::mod_flat!(win32);

pub fn init() -> anyhow::Result<()> {
	_ = fdlimit::raise_fd_limit();

	log::LOG_LEVEL.replace(<_>::from(std::env::var("YAZI_LOG").unwrap_or_default()));

	rustls::init()?;

	Ok(())
}
