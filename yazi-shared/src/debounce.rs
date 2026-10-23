use std::{pin::Pin, task::{Context, Poll}, time::Duration};

use futures::{FutureExt, Stream, StreamExt, stream::Fuse};
use pin_project::pin_project;
use tokio::time::{Sleep, sleep};

#[pin_project]
pub struct Debounce<S>
where
	S: Stream,
{
	stream:   Fuse<S>,
	interval: Duration,

	#[pin]
	sleep: Sleep,
	last:  Option<S::Item>,
}

impl<S> Debounce<S>
where
	S: Stream + Unpin,
{
	pub fn new(stream: S, interval: Duration) -> Self {
		Self { stream: stream.fuse(), interval, sleep: sleep(Duration::ZERO), last: None }
	}
}

impl<S> Stream for Debounce<S>
where
	S: Stream + Unpin,
{
	type Item = S::Item;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let mut me = self.project();

		while let Poll::Ready(next) = me.stream.poll_next_unpin(cx) {
			let Some(next) = next else { return Poll::Ready(me.last.take()) };
			*me.last = Some(next);
			me.sleep.as_mut().reset(sleep(*me.interval).deadline());
		}

		if me.last.is_some() && me.sleep.poll_unpin(cx).is_ready() {
			Poll::Ready(me.last.take())
		} else {
			Poll::Pending
		}
	}
}
