#![allow(clippy::option_map_unit_fn)]
pub mod body;
mod client;
mod payload;
mod pubsub;
mod sendable;
mod server;
mod state;

pub use client::*;
pub use payload::*;
pub use pubsub::*;
pub use sendable::*;
use server::*;
pub use state::*;

pub fn init() {
	let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

	// Client
	ID.init(yazi_shared::timestamp_us());
	PEERS.with(Default::default);
	QUEUE.init(tx);

	// Server
	CLIENTS.with(Default::default);
	STATE.with(Default::default);

	// Pubsub
	LOCAL.with(Default::default);
	REMOTE.with(Default::default);

	Client::serve(rx);
}
