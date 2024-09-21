use std::{pin::Pin, task::{Context, Poll}, time::Duration};

use futures::{FutureExt, Stream, StreamExt};
use tokio::time::{Instant, Sleep, sleep};

pub struct Debounce<S>
where
	S: Stream,
{
	stream:   S,
	interval: Duration,

	sleep: Sleep,
	last:  Option<S::Item>,
}

impl<S> Debounce<S>
where
	S: Stream + Unpin,
{
	pub fn new(stream: S, interval: Duration) -> Debounce<S> {
		Self { stream, interval, sleep: sleep(Duration::ZERO), last: None }
	}
}

impl<S> Stream for Debounce<S>
where
	S: Stream + Unpin,
{
	type Item = S::Item;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let (mut stream, interval, mut sleep, last) = unsafe {
			let me = self.get_unchecked_mut();
			(Pin::new(&mut me.stream), me.interval, Pin::new_unchecked(&mut me.sleep), &mut me.last)
		};

		if sleep.poll_unpin(cx).is_ready() {
			if let Some(last) = last.take() {
				return Poll::Ready(Some(last));
			}
		}

		while let Poll::Ready(next) = stream.poll_next_unpin(cx) {
			match next {
				Some(next) => {
					*last = Some(next);
				}
				None if last.is_none() => {
					return Poll::Ready(None);
				}
				None => {
					sleep.reset(Instant::now());
					return Poll::Pending;
				}
			}
		}

		sleep.reset(Instant::now() + interval);
		Poll::Pending
	}
}
