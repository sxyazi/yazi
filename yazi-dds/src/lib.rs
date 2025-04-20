#![allow(clippy::option_map_unit_fn)]

yazi_macro::mod_pub!(body);

yazi_macro::mod_flat!(client payload pubsub pump sendable server state stream);

pub fn init() {
	let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

	// Client
	ID.init(yazi_boot::ARGS.client_id.unwrap_or(yazi_shared::Id::unique()));
	PEERS.with(<_>::default);
	QUEUE_TX.init(tx);
	QUEUE_RX.init(rx);

	// Server
	CLIENTS.with(<_>::default);
	STATE.with(<_>::default);

	// Pubsub
	LOCAL.with(<_>::default);
	REMOTE.with(<_>::default);

	// Env
	unsafe {
		if let Some(s) = std::env::var("YAZI_ID").ok().filter(|s| !s.is_empty()) {
			std::env::set_var("YAZI_PID", s);
		}
		std::env::set_var("YAZI_ID", ID.to_string());
		std::env::set_var(
			"YAZI_LEVEL",
			(std::env::var("YAZI_LEVEL").unwrap_or_default().parse().unwrap_or(0u16) + 1).to_string(),
		);
	}
}

pub fn serve() {
	Pump::serve();
	Client::serve();
}

pub async fn shutdown() { Pump::shutdown().await; }
