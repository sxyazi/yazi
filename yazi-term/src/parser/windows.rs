use windows_sys::Win32::System::Console;

use crate::{Dimension, event::Event, parser::Parser};

impl Parser {
	pub(crate) fn parse_input_records(&mut self, records: &[Console::INPUT_RECORD]) {
		for record in records {
			match record.EventType as u32 {
				Console::KEY_EVENT => {
					let event = unsafe { record.Event.KeyEvent };
					let ch = unsafe { event.uChar.AsciiChar } as u8;

					if event.bKeyDown != 0 && ch != 0 {
						self.parse(&[ch]);
					}
				}
				Console::WINDOW_BUFFER_SIZE_EVENT => {
					let event = unsafe { record.Event.WindowBufferSizeEvent };
					if event.dwSize.X <= 0 || event.dwSize.Y <= 0 {
						continue;
					}

					self.events.push_back(Event::Resize(Dimension {
						rows:   event.dwSize.Y as u16,
						cols:   event.dwSize.X as u16,
						width:  0,
						height: 0,
					}));
				}
				_ => {}
			}
		}

		self.flush();
	}
}
