use std::{io, ops::Deref};

use anyhow::Result;
use ratatui_core::{buffer::Buffer, layout::Rect, terminal::{CompletedFrame, Frame, Terminal}};
use tokio::task::JoinHandle;
use yazi_config::YAZI;
use yazi_emulator::{EMULATOR, Probe};
use yazi_macro::writef;
use yazi_proxy::AppProxy;
use yazi_shim::cell::SyncCell;
use yazi_term::{TERM, event::{Event, KeyEventKind}, stream::EventStream};
use yazi_tty::{TTY, TtyWriter, sequence::{DisableBracketedPaste, DisableColorSchemeUpdates, DisableDrag, DisableDrop, DisableFocusChange, DisableMouseCapture, DisablePasteEvents, EnableBracketedPaste, EnableColorSchemeUpdates, EnableDrag, EnableDrop, EnableFocusChange, EnableMouseCapture, EnablePasteEvents, EnterAlternateScreen, If, LeaveAlternateScreen, PopKeyboardFlags, PushKeyboardFlags, RestoreCursorStyle, SetTitle, ShowCursor}};

use crate::{RatermBackend, RatermOption, RatermState};

pub static STATE: SyncCell<RatermState> = SyncCell::new(RatermState::default());

pub struct Raterm {
	inner:       Terminal<RatermBackend<TtyWriter<'static>>>,
	_stream:     EventStream,
	pub probe:   Probe,
	forwarder:   JoinHandle<()>,
	last_area:   Rect,
	last_buffer: Buffer,
}

impl Deref for Raterm {
	type Target = Terminal<RatermBackend<TtyWriter<'static>>>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Drop for Raterm {
	fn drop(&mut self) {
		self.forwarder.abort();
		Self::stop();
	}
}

impl Raterm {
	pub fn start() -> Result<Self> {
		TERM.setup()?;
		TERM.enter_raw_mode()?;

		let opt = RatermOption::default();
		STATE.set(RatermState::new(&opt));

		let mut stream = EventStream::from(&*TERM);
		writef!(
			TTY.writer(),
			"{EnterAlternateScreen}{EnableBracketedPaste}{EnableFocusChange}{EnableColorSchemeUpdates}{}{}{}{}{EnablePasteEvents}",
			PushKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
				| PushKeyboardFlags::REPORT_ALTERNATE_KEYS
				| PushKeyboardFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
				| PushKeyboardFlags::REPORT_ASSOCIATED_TEXT,
			EnableDrag(""),
			EnableDrop(&["text/uri-list"]),
			If(opt.mouse, EnableMouseCapture),
		)?;

		let mut inner = Terminal::new(RatermBackend::new(TTY.writer()))?;
		inner.hide_cursor()?;
		inner.clear()?;
		inner.flush()?;

		Ok(Self {
			inner,
			forwarder: Self::spawn(&mut stream),
			_stream: stream,
			probe: Probe::start()?,
			last_area: Default::default(),
			last_buffer: Default::default(),
		})
	}

	pub fn stop() {
		let emu = EMULATOR.load();
		let state = STATE.get();

		_ = writef!(
			TTY.writer(),
			"{}{PopKeyboardFlags}{DisableDrop}{DisablePasteEvents}{DisableDrag}{}{}{DisableColorSchemeUpdates}{DisableFocusChange}{DisableBracketedPaste}{LeaveAlternateScreen}{ShowCursor}",
			If(state.mouse, DisableMouseCapture),
			RestoreCursorStyle { blink: emu.cursor_blink, shape: emu.cursor_shape },
			If(state.title, SetTitle("")),
		);

		TERM.source.wake().ok();
		TERM.restorer.restore(&TTY);
	}

	fn spawn(stream: &mut EventStream) -> JoinHandle<()> {
		let mut rx = stream.take().unwrap();
		tokio::spawn(async move {
			loop {
				match rx.recv().await {
					Some(Ok(event)) => match event {
						Event::Key(key) if key.kind == KeyEventKind::Release => continue,
						Event::Mouse(mouse) if !YAZI.mgr.mouse_events.get().contains(mouse.kind.into()) => {
							continue;
						}
						_ => yazi_shared::event::Event::Term(event).emit(),
					},
					Some(Err(_)) => {
						AppProxy::quit(Default::default());
						break;
					}
					None => break,
				}
			}
		})
	}

	pub fn draw(&mut self, f: impl FnOnce(&mut Frame)) -> io::Result<CompletedFrame<'_>> {
		let last = self.inner.draw(f)?;

		self.last_area = last.area;
		self.last_buffer = last.buffer.clone();
		Ok(last)
	}

	pub fn draw_partial(&mut self, f: impl FnOnce(&mut Frame)) -> io::Result<CompletedFrame<'_>> {
		self.inner.draw(|frame| {
			let buffer = frame.buffer_mut();
			for y in self.last_area.top()..self.last_area.bottom() {
				for x in self.last_area.left()..self.last_area.right() {
					buffer[(x, y)] = self.last_buffer[(x, y)].clone();
				}
			}

			f(frame);
		})
	}

	pub fn can_partial(&mut self) -> bool {
		self.inner.autoresize().is_ok() && self.last_area == self.inner.get_frame().area()
	}
}
