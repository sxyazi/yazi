use std::{mem, pin::Pin, task::{Context, Poll}, time::Duration};

use futures::{FutureExt, Stream, StreamExt};
use tokio::time::{sleep, Instant, Sleep};

pub struct StreamBuf<S>
where
	S: Stream,
{
	stream:   S,
	interval: Duration,

	sleep:   Sleep,
	pending: Vec<S::Item>,
}

impl<S> Unpin for StreamBuf<S> where S: Stream {}

impl<S> StreamBuf<S>
where
	S: Stream + Unpin,
{
	pub fn new(stream: S, interval: Duration) -> StreamBuf<S> {
		Self { stream, interval, sleep: sleep(Duration::ZERO), pending: Default::default() }
	}

	pub fn flush(&mut self) {
		let sleep = unsafe { Pin::new_unchecked(&mut self.sleep) };
		sleep.reset(Instant::now());
	}
}

impl<S> Stream for StreamBuf<S>
where
	S: Stream + Unpin,
{
	type Item = Vec<S::Item>;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let (mut stream, interval, mut sleep, pending) = unsafe {
			let me = self.get_unchecked_mut();
			(Pin::new(&mut me.stream), me.interval, Pin::new_unchecked(&mut me.sleep), &mut me.pending)
		};

		while let Poll::Ready(next) = stream.poll_next_unpin(cx) {
			match next {
				Some(next) => pending.push(next),
				None => {
					if pending.is_empty() {
						return Poll::Ready(None);
					}
					break;
				}
			}
		}

		if pending.is_empty() {
			return Poll::Pending;
		}

		match sleep.poll_unpin(cx) {
			Poll::Ready(_) => {
				sleep.reset(Instant::now() + interval);
				Poll::Ready(Some(mem::take(pending)))
			}
			Poll::Pending => Poll::Pending,
		}
	}
}
