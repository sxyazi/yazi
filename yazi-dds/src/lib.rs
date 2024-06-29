#![allow(clippy::option_map_unit_fn)]
pub mod body;
mod client;
mod payload;
mod pubsub;
mod pump;
mod sendable;
mod server;
mod state;
mod stream;

pub use client::*;
pub use payload::*;
pub use pubsub::*;
pub use pump::*;
pub use sendable::*;
use server::*;
pub use state::*;
use stream::*;

#[cfg(unix)]
pub static USERS_CACHE: yazi_shared::RoCell<uzers::UsersCache> = yazi_shared::RoCell::new();

pub fn init() {
	let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

	// Client
	ID.init(yazi_shared::timestamp_us());
	PEERS.with(Default::default);
	QUEUE_TX.init(tx);
	QUEUE_RX.init(rx);

	// Server
	CLIENTS.with(Default::default);
	STATE.with(Default::default);

	// Pubsub
	LOCAL.with(Default::default);
	REMOTE.with(Default::default);

	#[cfg(unix)]
	USERS_CACHE.with(Default::default);

	// Env
	if let Some(s) = std::env::var("YAZI_ID").ok().filter(|s| !s.is_empty()) {
		std::env::set_var("YAZI_PID", s);
	}
	std::env::set_var("YAZI_ID", ID.to_string());
	std::env::set_var(
		"YAZI_LEVEL",
		(std::env::var("YAZI_LEVEL").unwrap_or_default().parse().unwrap_or(0u16) + 1).to_string(),
	);
}

pub fn serve() {
	Pump::serve();
	Client::serve();
}

pub async fn shutdown() { Pump::shutdown().await; }
