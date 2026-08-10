use anyhow::anyhow;

pub(super) fn init() -> anyhow::Result<()> {
	::rustls::crypto::ring::default_provider()
		.install_default()
		.map_err(|_| anyhow!("Failed to install ring crypto provider"))
}
