use tokio::sync::mpsc;

use super::Event;

pub(super) struct EventTx {
	pub(super) high:   mpsc::UnboundedSender<Event>,
	pub(super) normal: mpsc::UnboundedSender<Event>,
}

pub struct EventRx {
	pub(super) high:   mpsc::UnboundedReceiver<Event>,
	pub(super) normal: mpsc::UnboundedReceiver<Event>,
}

impl EventRx {
	pub async fn recv(&mut self) -> Option<Event> {
		tokio::select! {
			biased;
			event = self.high.recv() => event,
			event = self.normal.recv() => event,
		}
	}

	pub fn try_recv(&mut self) -> Result<Event, mpsc::error::TryRecvError> {
		self.high.try_recv().or_else(|_| self.normal.try_recv())
	}
}
