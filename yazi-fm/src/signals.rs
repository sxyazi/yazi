use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use futures::StreamExt;
use tokio::{select, sync::{mpsc, oneshot}};
use yazi_config::YAZI;
use yazi_shared::event::Event;

pub(super) struct Signals {
	tx: mpsc::UnboundedSender<(bool, Option<oneshot::Sender<()>>)>,
}

impl Signals {
	pub(super) fn start() -> Result<Self> {
		let (tx, rx) = mpsc::unbounded_channel();
		Self::spawn(rx)?;

		Ok(Self { tx })
	}

	pub(super) fn stop(&mut self, cb: Option<oneshot::Sender<()>>) { self.tx.send((false, cb)).ok(); }

	pub(super) fn resume(&mut self, cb: Option<oneshot::Sender<()>>) {
		self.tx.send((true, cb)).ok();
	}

	#[cfg(unix)]
	#[inline]
	fn handle_sys(n: libc::c_int) -> bool {
		use libc::{SIGCONT, SIGHUP, SIGINT, SIGQUIT, SIGSTOP, SIGTERM, SIGTSTP};
		use tracing::error;
		use yazi_proxy::{AppProxy, HIDER};

		match n {
			SIGINT => { /* ignored */ }
			SIGQUIT | SIGHUP | SIGTERM => {
				Event::Quit(Default::default()).emit();
				return false;
			}
			SIGTSTP => {
				tokio::spawn(async move {
					AppProxy::stop().await;
					if unsafe { libc::kill(0, SIGSTOP) } != 0 {
						error!("Failed to stop the process:\n{}", std::io::Error::last_os_error());
						Event::Quit(Default::default()).emit();
					}
				});
			}
			SIGCONT if HIDER.try_acquire().is_ok() => AppProxy::resume(),
			_ => {}
		}
		true
	}

	#[cfg(windows)]
	#[inline]
	fn handle_sys(_: ()) -> bool { unreachable!() }

	#[inline]
	fn handle_term(event: CrosstermEvent) {
		match event {
			CrosstermEvent::Key(key @ KeyEvent { kind: KeyEventKind::Press, .. }) => {
				Event::Key(key).emit()
			}
			CrosstermEvent::Mouse(mouse) => {
				if YAZI.mgr.mouse_events.contains(mouse.kind.into()) {
					Event::Mouse(mouse).emit();
				}
			}
			CrosstermEvent::Paste(str) => Event::Paste(str).emit(),
			CrosstermEvent::Resize(..) => Event::Resize.emit(),
			_ => {}
		}
	}

	fn spawn(mut rx: mpsc::UnboundedReceiver<(bool, Option<oneshot::Sender<()>>)>) -> Result<()> {
		#[cfg(unix)]
		use libc::{SIGCONT, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGTSTP};

		#[cfg(unix)]
		let mut sys = signal_hook_tokio::Signals::new([
			// Interrupt signals (Ctrl-C, Ctrl-\)
			SIGINT, SIGQUIT, //
			// Hangup signal (Terminal closed)
			SIGHUP, //
			// Termination signal (kill)
			SIGTERM, //
			// Job control signals (Ctrl-Z, fg/bg)
			SIGTSTP, SIGCONT,
		])?;
		#[cfg(windows)]
		let mut sys = tokio_stream::empty();

		let mut term = Some(EventStream::new());

		tokio::spawn(async move {
			loop {
				if let Some(t) = &mut term {
					select! {
						biased;
						Some((state, mut callback)) = rx.recv() => {
							term = term.filter(|_| state);
							callback.take().map(|cb| cb.send(()));
						},
						Some(n) = sys.next() => if !Self::handle_sys(n) { return },
						Some(Ok(e)) = t.next() => Self::handle_term(e)
					}
				} else {
					select! {
						biased;
						Some((state, mut callback)) = rx.recv() => {
							term = state.then(EventStream::new);
							callback.take().map(|cb| cb.send(()));
						},
						Some(n) = sys.next() => if !Self::handle_sys(n) { return },
					}
				}
			}
		});

		Ok(())
	}
}
