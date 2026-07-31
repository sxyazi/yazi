use std::{io, time::Duration};

use crate::{Timeout, event::Event, source::EventSource};

impl<'a> EventSource<'a> {
	pub fn wake(&self) -> io::Result<()> { self.waker.wake() }

	pub fn try_poll<F>(&self, timeout: Option<Duration>, mut filter: F) -> io::Result<Event>
	where
		F: FnMut(&Event) -> bool,
	{
		let timeout = Timeout::new(timeout);

		loop {
			let mut parser = self.parser.lock();
			if let Some(i) = parser.events.iter().position(&mut filter) {
				return Ok(parser.events.remove(i).unwrap());
			}

			drop(parser);
			if timeout.elapsed() {
				return Err(io::ErrorKind::TimedOut.into());
			}

			match self.try_fill(timeout) {
				Ok(()) => {}
				Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
				Err(e) => return Err(e),
			}
		}
	}

	pub(crate) fn drain(&self) -> io::Result<()> {
		let result = self.waker.drain();
		self.parser.lock().drain();
		result
	}
}

pub(super) fn min_timeout(a: Option<Duration>, b: Option<Duration>) -> Option<Duration> {
	a.into_iter().chain(b).min()
}
