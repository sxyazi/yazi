use std::{pin::Pin, sync::Arc, task::Poll};

use tokio::sync::oneshot;

use crate::{Packet, Session};

pub struct Receiver {
	rx:       oneshot::Receiver<Packet<'static>>,
	received: bool,

	session: Arc<Session>,
	id:      u32,
}

impl Drop for Receiver {
	fn drop(&mut self) {
		if !self.received {
			self.session.callback.lock().remove(&self.id);
		}
	}
}

impl Receiver {
	pub(crate) fn new(
		session: &Arc<Session>,
		id: u32,
		rx: oneshot::Receiver<Packet<'static>>,
	) -> Self {
		Self { rx, received: false, session: session.clone(), id }
	}
}

impl Future for Receiver {
	type Output = Result<Packet<'static>, oneshot::error::RecvError>;

	fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
		let me = self.get_mut();
		match Pin::new(&mut me.rx).poll(cx) {
			Poll::Ready(Ok(packet)) => {
				me.received = true;
				Poll::Ready(Ok(packet))
			}
			Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
			Poll::Pending => Poll::Pending,
		}
	}
}
