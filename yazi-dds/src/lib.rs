yazi_macro::mod_pub!(ember);

yazi_macro::mod_flat!(client payload pubsub pump server state stream subscription subscriptions);

pub fn init() -> tokio::sync::mpsc::UnboundedReceiver<String> {
	let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
	QUEUE_TX.init(tx);

	// Env
	unsafe {
		if let Some(s) = std::env::var("YAZI_ID").ok().filter(|s| !s.is_empty()) {
			std::env::set_var("YAZI_PID", s);
		}
		std::env::set_var("YAZI_ID", yazi_boot::BOOT.id.to_string());
		std::env::set_var(
			"YAZI_LEVEL",
			(std::env::var("YAZI_LEVEL").unwrap_or_default().parse().unwrap_or(0u16) + 1).to_string(),
		);
	}

	rx
}

pub fn serve() {
	let rx = init();
	Pump::serve();
	Client::serve(rx);
}

pub async fn shutdown() { Pump::shutdown().await; }
