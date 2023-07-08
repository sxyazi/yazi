use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream};
use futures::StreamExt;
use libc::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use tokio::{select, sync::{mpsc::{self, Receiver, Sender}, oneshot}, task::JoinHandle};

use crate::core::Event;

pub struct Signals {
	pub tx: Sender<Event>,
	pub rx: Receiver<Event>,

	term_stop_tx: Option<oneshot::Sender<()>>,
	term_stop_rx: Option<oneshot::Receiver<()>>,
}

impl Signals {
	pub fn start() -> Result<Self> {
		let (tx, rx) = mpsc::channel(500);
		let (term_tx, term_rx) = oneshot::channel();

		let mut signals =
			Self { tx: tx.clone(), rx, term_stop_tx: Some(term_tx), term_stop_rx: Some(term_rx) };

		signals.spawn_system_task()?;
		signals.spawn_crossterm_task();

		Event::init(tx);
		Ok(signals)
	}

	fn spawn_system_task(&self) -> Result<JoinHandle<()>> {
		let tx = self.tx.clone();
		let mut signals = signal_hook_tokio::Signals::new([SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;

		Ok(tokio::spawn(async move {
			while let Some(signal) = signals.next().await {
				match signal {
					SIGHUP | SIGTERM | SIGINT | SIGQUIT => {
						if tx.send(Event::Quit).await.is_err() {
							break;
						}
					}
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
							CrosstermEvent::Key(key) => Event::Key(key),
							CrosstermEvent::Resize(cols, rows) => Event::Resize(cols, rows),
							_ => continue,
						};
						if tx.send(event).await.is_err() {
							break;
						}
					}
				}
			}
		})
	}

	pub fn stop_term(&mut self, state: bool) {
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
}
