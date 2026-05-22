use std::{io::{self, Write}, ops::Deref};

use anyhow::Result;
use ratatui::{CompletedFrame, Frame, Terminal, buffer::Buffer, layout::Rect};
use yazi_config::YAZI;
use yazi_emulator::{Emulator, Mux, TMUX};
use yazi_macro::writef;
use yazi_shim::cell::SyncCell;
use yazi_term::{TERM, event::{Event, KeyEventKind}, sequence::{DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste, EnableFocusChange, EnableMouseCapture, EnterAlternateScreen, If, LeaveAlternateScreen, PopKeyboardFlags, PushKeyboardFlags, RequestCursorBlink, RequestCursorStyle, RequestDA1, RequestKeyboardFlags, RestoreBackground, RestoreCursorStyle, SetBackground, SetTitle, ShowCursor}, stream::EventStream};
use yazi_tty::{TTY, TtyWriter};

use crate::{RatermBackend, RatermOption, RatermState};

pub static STATE: SyncCell<RatermState> = SyncCell::new(RatermState::default());

pub struct Raterm {
	inner:       Terminal<RatermBackend<TtyWriter<'static>>>,
	stream:      EventStream,
	last_area:   Rect,
	last_buffer: Buffer,
}

impl Deref for Raterm {
	type Target = Terminal<RatermBackend<TtyWriter<'static>>>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Drop for Raterm {
	fn drop(&mut self) { Self::stop(); }
}

impl Raterm {
	pub fn start() -> Result<Self> {
		let opt = RatermOption::default();
		let mut term = Self {
			inner:       Terminal::new(RatermBackend::new(TTY.writer()))?,
			stream:      EventStream::from(&*TERM),
			last_area:   Default::default(),
			last_buffer: Default::default(),
		};

		TERM.setup()?;
		TERM.enter_raw_mode()?;
		static FIRST: SyncCell<bool> = SyncCell::new(false);
		if FIRST.replace(true) && yazi_emulator::TMUX.get() {
			yazi_emulator::Mux::tmux_passthrough();
		}

		writef!(
			TTY.writer(),
			"{}{RequestCursorStyle}{RequestCursorBlink}{RequestKeyboardFlags}{RequestDA1}{}{}{EnableBracketedPaste}{EnableFocusChange}{}",
			If(!TMUX.get(), EnterAlternateScreen),
			If(TMUX.get(), EnterAlternateScreen),
			SetBackground(&opt.bg),
			If(opt.mouse, EnableMouseCapture),
		)?;

		let resp = Emulator::read_until_da1();
		Mux::tmux_drain()?;

		STATE.set(RatermState::new(&resp, &opt));
		if STATE.get().csi_u {
			write!(
				TTY.writer(),
				"{}",
				PushKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES | PushKeyboardFlags::REPORT_ALTERNATE_KEYS,
			)?;
		}

		term.inner.hide_cursor()?;
		term.inner.clear()?;
		term.inner.flush()?;
		term.spawn();
		Ok(term)
	}

	pub fn stop() {
		let state = STATE.get();

		_ = writef!(
			TTY.writer(),
			"{}{}{}{}{}{DisableFocusChange}{DisableBracketedPaste}{LeaveAlternateScreen}{ShowCursor}",
			If(state.mouse, DisableMouseCapture),
			If(state.bg, RestoreBackground),
			If(state.csi_u, PopKeyboardFlags),
			RestoreCursorStyle { shape: state.cursor_shape, blink: state.cursor_blink },
			If(state.title, SetTitle("")),
		);

		TERM.source.wake().ok();
		TERM.restorer.restore(&TTY);
	}

	fn spawn(&mut self) {
		let mut rx = self.stream.take().unwrap();

		tokio::spawn(async move {
			while let Some(Ok(event)) = rx.recv().await {
				match event {
					Event::Key(key) if key.kind != KeyEventKind::Press => continue,
					Event::Mouse(mouse) if !YAZI.mgr.mouse_events.get().contains(mouse.kind.into()) => {
						continue;
					}
					_ => yazi_shared::event::Event::Term(event).emit(),
				}
			}
		});
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
					let mut cell = self.last_buffer[(x, y)].clone();
					cell.skip = false;
					buffer[(x, y)] = cell;
				}
			}

			f(frame);
		})
	}

	pub fn can_partial(&mut self) -> bool {
		self.inner.autoresize().is_ok() && self.last_area == self.inner.get_frame().area()
	}
}
