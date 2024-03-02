use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use futures::StreamExt;
use tokio::{select, sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use yazi_shared::event::Event;

pub(super) struct Signals {
	tx:     mpsc::UnboundedSender<Event>,
	pub rx: mpsc::UnboundedReceiver<Event>,
	ct:     CancellationToken,
}

impl Signals {
	pub(super) fn start() -> Result<Self> {
		let (tx, rx) = mpsc::unbounded_channel();
		let ct = CancellationToken::new();
		let mut signals = Self { tx: tx.clone(), rx, ct };

		Event::init(tx);
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
		use yazi_scheduler::BLOCKER;

		let mut signals = signal_hook_tokio::Signals::new([
			// Terminating signals
			SIGHUP, SIGTERM, SIGQUIT, SIGINT, //
			// Job control signals
			SIGCONT,
		])?;

		let tx = self.tx.clone();
		Ok(tokio::spawn(async move {
			while let Some(signal) = signals.next().await {
				if BLOCKER.try_acquire().is_err() {
					continue;
				}

				match signal {
					SIGHUP | SIGTERM | SIGQUIT | SIGINT => {
						if tx.send(Event::Quit(Default::default())).is_err() {
							break;
						}
					}
					SIGCONT => yazi_proxy::App::resume(),
					_ => {}
				}
			}
		}))
	}

	fn spawn_crossterm_task(&mut self) -> JoinHandle<()> {
		let mut reader = EventStream::new();
		let (tx, ct) = (self.tx.clone(), self.ct.clone());

		tokio::spawn(async move {
			loop {
				select! {
					_ = ct.cancelled() => break,
					Some(Ok(event)) = reader.next() => {
						let event = match event {
							// We need to check key event kind;
							// otherwise event will be dispatched twice.
							CrosstermEvent::Key(key @ KeyEvent { kind: KeyEventKind::Press, .. }) => Event::Key(key),
							CrosstermEvent::Paste(str) => Event::Paste(str),
							CrosstermEvent::Resize(..) => Event::Resize,
							_ => continue,
						};
						if tx.send(event).is_err() {
							break;
						}
					}
				}
			}
		})
	}
}
