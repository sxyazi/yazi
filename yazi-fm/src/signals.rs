use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use futures::StreamExt;
use tokio::{select, sync::mpsc};
use yazi_config::YAZI;
use yazi_shared::{CompletionToken, event::Event};

pub(super) struct Signals {
	pub(super) tx: mpsc::UnboundedSender<(bool, CompletionToken)>,
}

impl Signals {
	pub(super) fn start() -> Result<Self> {
		let (tx, rx) = mpsc::unbounded_channel();
		Self::spawn(rx)?;

		Ok(Self { tx })
	}

	#[cfg(unix)]
	fn handle_sys(n: libc::c_int) -> bool {
		use libc::{SIGCONT, SIGHUP, SIGINT, SIGQUIT, SIGSTOP, SIGTERM, SIGTSTP};
		use tracing::error;
		use yazi_proxy::AppProxy;
		use yazi_term::YIELD_TO_SUBPROCESS;

		match n {
			SIGINT => { /* ignored */ }
			SIGQUIT | SIGHUP | SIGTERM => {
				AppProxy::quit(Default::default());
				return false;
			}
			SIGTSTP => {
				tokio::spawn(async move {
					AppProxy::stop().await;
					if unsafe { libc::kill(0, SIGSTOP) } != 0 {
						error!("Failed to stop the process:\n{}", std::io::Error::last_os_error());
						AppProxy::quit(Default::default());
					}
				});
			}
			SIGCONT if YIELD_TO_SUBPROCESS.try_acquire().is_ok() => _ = tokio::spawn(AppProxy::resume()),
			_ => {}
		}
		true
	}

	#[cfg(windows)]
	#[inline]
	fn handle_sys(_: ()) -> bool { unreachable!() }

	fn handle_term(event: CrosstermEvent) {
		match event {
			CrosstermEvent::Key(key @ KeyEvent { kind: KeyEventKind::Press, .. }) => {
				Event::Key(key).emit()
			}
			CrosstermEvent::Mouse(mouse) => {
				if YAZI.mgr.mouse_events.get().contains(mouse.kind.into()) {
					Event::Mouse(mouse).emit();
				}
			}
			CrosstermEvent::Resize(..) => Event::Resize.emit(),
			CrosstermEvent::FocusGained => Event::Focus.emit(),
			CrosstermEvent::Paste(str) => Event::Paste(str).emit(),
			_ => {}
		}
	}

	fn spawn(mut rx: mpsc::UnboundedReceiver<(bool, CompletionToken)>) -> Result<()> {
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
						Some((state, token)) = rx.recv() => {
							term = term.filter(|_| state);
							token.complete(true);
						},
						Some(n) = sys.next() => if !Self::handle_sys(n) { return },
						Some(Ok(e)) = t.next() => Self::handle_term(e)
					}
				} else {
					select! {
						biased;
						Some((state, token)) = rx.recv() => {
							term = state.then(EventStream::new);
							token.complete(true);
						},
						Some(n) = sys.next() => if !Self::handle_sys(n) { return },
					}
				}
			}
		});

		Ok(())
	}
}
