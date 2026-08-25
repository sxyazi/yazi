use std::{io, pin::Pin, sync::Arc, task::{Context, Poll}, thread};

use futures::Stream;
use tokio::sync::mpsc;

use crate::{event::Event, source::EventSource, terminal::Terminal};

pub struct EventStream {
	rx:     Option<mpsc::UnboundedReceiver<io::Result<Event>>>,
	source: Arc<EventSource<'static>>,
	worker: Option<thread::JoinHandle<()>>,
}

impl Drop for EventStream {
	fn drop(&mut self) {
		self.source.wake().ok();
		if let Some(worker) = self.worker.take() {
			worker.join().ok();
		}
		self.source.drain().ok();
	}
}

impl From<&'static Terminal<'_>> for EventStream {
	fn from(value: &'static Terminal<'_>) -> Self { Self::new(value.source.clone(), |_| true) }
}

impl EventStream {
	fn new<F>(source: Arc<EventSource<'static>>, filter: F) -> Self
	where
		F: Fn(&Event) -> bool + Send + 'static,
	{
		let source_ = source.clone();
		let (tx, rx) = mpsc::unbounded_channel();

		let worker = thread::spawn(move || {
			loop {
				match source_.try_poll(None, |_| true) {
					Ok(event) => {
						if !(filter)(&event) {
							continue;
						}
						if tx.send(Ok(event)).is_err() {
							break;
						}
					}
					// try_poll() already handles Interrupted, this is defensive.
					Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
					// event source was gracefully stopped, stop the stream as well.
					Err(e) if e.kind() == io::ErrorKind::ConnectionAborted => break,
					Err(e) => {
						tx.send(Err(e)).ok();
						break;
					}
				}
			}
		});

		Self { rx: Some(rx), source, worker: Some(worker) }
	}

	pub fn take(&mut self) -> Option<mpsc::UnboundedReceiver<io::Result<Event>>> { self.rx.take() }
}

impl Stream for EventStream {
	type Item = io::Result<Event>;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		match &mut Pin::into_inner(self).rx {
			Some(rx) => rx.poll_recv(cx),
			None => Poll::Ready(None),
		}
	}
}
