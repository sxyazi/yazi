use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use futures::StreamExt;
use tokio::{select, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use yazi_config::MANAGER;
use yazi_shared::event::Event;

pub(super) struct Signals {
	ct: CancellationToken,
}

impl Signals {
	pub(super) fn start() -> Result<Self> {
		let mut signals = Self { ct: CancellationToken::new() };

		signals.spawn_system_task()?;
		signals.spawn_crossterm_task();

		Ok(signals)
	}

	pub(super) fn stop(&mut self) {
		if !self.ct.is_cancelled() {
			self.ct.cancel();
		}
	}

	pub(super) fn resume(&mut self) {
		if self.ct.is_cancelled() {
			self.ct = CancellationToken::new();
			self.spawn_crossterm_task();
		}
	}

	#[cfg(windows)]
	fn spawn_system_task(&self) -> Result<()> { Ok(()) }

	#[cfg(unix)]
	fn spawn_system_task(&self) -> Result<JoinHandle<()>> {
		use libc::{SIGCONT, SIGHUP, SIGINT, SIGQUIT, SIGTERM};
		use yazi_proxy::{AppProxy, HIDER};

		let mut signals = signal_hook_tokio::Signals::new([
			// Terminating signals
			SIGHUP, SIGTERM, SIGQUIT, SIGINT, //
			// Job control signals
			SIGCONT,
		])?;

		Ok(tokio::spawn(async move {
			while let Some(signal) = signals.next().await {
				match signal {
					SIGHUP | SIGTERM | SIGQUIT => {
						Event::Quit(Default::default()).emit();
					}
					SIGCONT if HIDER.try_acquire().is_ok() => AppProxy::resume(),
					_ => {}
				}
			}
		}))
	}

	fn spawn_crossterm_task(&mut self) -> JoinHandle<()> {
		let mut reader = EventStream::new();
		let ct = self.ct.clone();

		tokio::spawn(async move {
			loop {
				select! {
					_ = ct.cancelled() => break,
					Some(Ok(event)) = reader.next() => {
						 match event {
							CrosstermEvent::Key(key @ KeyEvent { kind: KeyEventKind::Press, .. }) => Event::Key(key).emit(),
							CrosstermEvent::Mouse(mouse) => {
								if MANAGER.mouse_events.contains(mouse.kind.into()) {
									Event::Mouse(mouse).emit();
								}
							},
							CrosstermEvent::Paste(str) => Event::Paste(str).emit(),
							CrosstermEvent::Resize(..) => Event::Resize.emit(),
							_ => {},
						}
					}
				}
			}
		})
	}
}
