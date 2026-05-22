use std::{io, time::Duration};

use crate::{Timeout, event::Event, source::EventSource};

impl<'a> EventSource<'a> {
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
			match self.try_fill(timeout) {
				Ok(()) => {}
				Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
				Err(e) => return Err(e),
			}

			if timeout.elapsed() {
				return Err(io::Error::from(io::ErrorKind::TimedOut));
			}
		}
	}
}
