use tokio::sync::mpsc;
use yazi_shim::cell::RoCell;

use super::{ActionCow, EventRx, channel::EventTx};

static TX: RoCell<EventTx> = RoCell::new();
static RX: RoCell<EventRx> = RoCell::new();

#[derive(Debug)]
pub enum Event {
	Call(ActionCow),
	Seq(Vec<ActionCow>),
	Term(yazi_term::event::Event),
}

impl Event {
	#[inline]
	pub(crate) fn init() {
		let (high_tx, high_rx) = mpsc::unbounded_channel();
		let (normal_tx, normal_rx) = mpsc::unbounded_channel();

		TX.init(EventTx { high: high_tx, normal: normal_tx });
		RX.init(EventRx { high: high_rx, normal: normal_rx });
	}

	#[inline]
	pub fn take() -> EventRx { RX.drop() }

	#[inline]
	pub fn emit(self) { TX.normal.send(self).ok(); }

	#[inline]
	pub fn preempt(self) { TX.high.send(self).ok(); }
}
