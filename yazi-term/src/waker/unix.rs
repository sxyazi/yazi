use std::{io::{self, Read, Write}, ops::Deref, os::unix::net::UnixStream};

#[derive(Debug)]
pub(crate) struct Waker {
	reader: UnixStream,
	writer: UnixStream,
}

impl Deref for Waker {
	type Target = UnixStream;

	fn deref(&self) -> &Self::Target { &self.reader }
}

impl Waker {
	pub(crate) fn new() -> io::Result<Self> {
		let (reader, writer) = UnixStream::pair()?;
		reader.set_nonblocking(true)?;
		writer.set_nonblocking(true)?;

		Ok(Self { reader, writer })
	}

	pub fn wake(&self) -> io::Result<()> { Write::write_all(&mut &self.writer, &[0]) }

	pub(crate) fn drain(&self) -> io::Result<()> {
		loop {
			match Read::read(&mut &self.reader, &mut [0; 1024]) {
				Ok(0) => return Ok(()),
				Ok(_) => {}
				Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
				Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
				Err(e) => return Err(e),
			}
		}
	}
}
