use std::sync::{Arc, atomic::{AtomicU8, Ordering}};

use tokio::sync::mpsc;
use yazi_macro::impl_data_any;

use crate::data::Data;

type Reply = anyhow::Result<Data>;

#[derive(Clone, Debug)]
pub struct Replier {
	tx:    mpsc::UnboundedSender<Reply>,
	state: Arc<AtomicU8>,
}

impl_data_any!(Replier);

impl From<mpsc::UnboundedSender<Reply>> for Replier {
	fn from(tx: mpsc::UnboundedSender<Reply>) -> Self {
		Self { tx, state: Arc::new(AtomicU8::new(0)) }
	}
}

impl Replier {
	pub(super) fn claim(&self) {
		_ = self.state.compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed);
	}

	pub fn send(&self, result: Reply) -> Result<(), mpsc::error::SendError<Reply>> {
		let state = self.state.swap(2, Ordering::Relaxed);
		if state == 2 { Err(mpsc::error::SendError(result)) } else { self.tx.send(result) }
	}

	pub fn reply_if_unclaimed(&self, result: Reply) -> bool {
		self.state.compare_exchange(0, 2, Ordering::Relaxed, Ordering::Relaxed).is_ok()
			&& self.tx.send(result).is_ok()
	}
}
