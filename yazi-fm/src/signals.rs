use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use futures::StreamExt;
use tokio::{select, sync::{mpsc::{self, UnboundedReceiver, UnboundedSender}, oneshot}, task::JoinHandle};
use yazi_core::Event;

pub(super) struct Signals {
	tx: UnboundedSender<Event>,
	rx: UnboundedReceiver<Event>,

	term_stop_tx: Option<oneshot::Sender<()>>,
	term_stop_rx: Option<oneshot::Receiver<()>>,
}

impl Signals {
	pub(super) fn start() -> Result<Self> {
		let (tx, rx) = mpsc::unbounded_channel();
		let (term_tx, term_rx) = oneshot::channel();

		let mut signals =
			Self { tx: tx.clone(), rx, term_stop_tx: Some(term_tx), term_stop_rx: Some(term_rx) };

		signals.spawn_system_task()?;
		signals.spawn_crossterm_task();

		Event::init(tx);
		Ok(signals)
	}

	#[inline]
	pub(super) async fn recv(&mut self) -> Option<Event> { self.rx.recv().await }

	pub(super) fn stop_term(&mut self, state: bool) {
		if state == self.term_stop_tx.is_none() {
			return;
		}

		if let Some(tx) = self.term_stop_tx.take() {
			tx.send(()).ok();
		} else {
			let (tx, rx) = oneshot::channel();
			(self.term_stop_tx, self.term_stop_rx) = (Some(tx), Some(rx));
			self.spawn_crossterm_task();
		}
	}

	#[cfg(windows)]
	fn spawn_system_task(&self) -> Result<()> { Ok(()) }

	#[cfg(unix)]
	fn spawn_system_task(&self) -> Result<JoinHandle<()>> {
		use libc::{SIGCONT, SIGHUP, SIGINT, SIGQUIT, SIGTERM};
		use yazi_core::Ctx;

		let tx = self.tx.clone();
		let mut signals = signal_hook_tokio::Signals::new([
			// Terminating signals
			SIGHUP, SIGTERM, SIGQUIT, SIGINT, //
			// Job control signals
			SIGCONT,
		])?;

		Ok(tokio::spawn(async move {
			while let Some(signal) = signals.next().await {
				match signal {
					SIGHUP | SIGTERM | SIGQUIT | SIGINT => {
						if tx.send(Event::Quit(false)).is_err() {
							break;
						}
					}
					SIGCONT => Ctx::resume(),
					_ => {}
				}
			}
		}))
	}

	fn spawn_crossterm_task(&mut self) -> JoinHandle<()> {
		let tx = self.tx.clone();
		let mut stop_rx = self.term_stop_rx.take().unwrap();

		tokio::spawn(async move {
			let mut reader = EventStream::new();

			loop {
				select! {
					_ = &mut stop_rx => break,
					Some(Ok(event)) = reader.next() => {
						let event = match event {
							// We need to check key event kind;
							// otherwise event will be dispatched twice.
							CrosstermEvent::Key(key @ KeyEvent { kind: KeyEventKind::Press, .. }) => Event::Key(key),
							CrosstermEvent::Paste(str) => Event::Paste(str),
							CrosstermEvent::Resize(cols, rows) => Event::Resize(cols, rows),
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
