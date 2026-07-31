use std::{io, mem, os::windows::io::{AsRawHandle, RawHandle}};

use parking_lot::Mutex;
use windows_sys::Win32::{Foundation::{WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT}, System::{Console::{INPUT_RECORD, ReadConsoleInputA}, Threading::{INFINITE, WaitForMultipleObjects}}};
use yazi_tty::TtyReader;

use crate::{Timeout, parser::Parser, source::min_timeout, waker::Waker};

pub struct EventSource<'a> {
	pub(super) parser: Mutex<Parser>,
	reader:            TtyReader<'a>,
	pub(super) waker:  Waker,
}

impl<'a> EventSource<'a> {
	pub(crate) fn new(reader: TtyReader<'a>) -> io::Result<Self> {
		Ok(Self { reader, parser: Mutex::new(Parser::default()), waker: Waker::new()? })
	}

	pub(crate) fn try_fill(&self, timeout: Timeout) -> io::Result<()> {
		let reader = self.reader.lock();
		let duration = min_timeout(timeout.leftover(), self.parser.lock().leftover());
		let millis = duration
			.map(|dur| dur.as_nanos().div_ceil(1_000_000).min(u32::MAX as u128) as u32)
			.unwrap_or(INFINITE);

		let handles = [reader.as_raw_handle(), self.waker.as_raw_handle()];
		match unsafe { WaitForMultipleObjects(handles.len() as u32, handles.as_ptr(), 0, millis) } {
			// More input is ready.
			WAIT_OBJECT_0 => {}
			// Stop waiting for events.
			r if r == WAIT_OBJECT_0 + 1 => return Err(io::ErrorKind::ConnectionAborted.into()),
			// Parser or caller timeout expired.
			WAIT_TIMEOUT => {
				self.parser.lock().expire();
				return Ok(());
			}
			// An error occurred.
			WAIT_FAILED => return Err(io::Error::last_os_error()),
			// Unexpected return value.
			_ => return Err(io::Error::other("WaitForMultipleObjects returned unexpected value")),
		}

		let (buf, len) = read_console(reader.as_raw_handle())?;
		self.parser.lock().parse_input_records(&buf[..len]);

		Ok(())
	}
}

fn read_console(handle: RawHandle) -> io::Result<([INPUT_RECORD; 128], usize)> {
	let mut buf = [unsafe { mem::zeroed::<INPUT_RECORD>() }; 128];
	let mut len = 0u32;

	match unsafe { ReadConsoleInputA(handle, buf.as_mut_ptr(), buf.len() as u32, &mut len) } {
		0 => Err(io::Error::last_os_error()),
		_ => Ok((buf, len as usize)),
	}
}
