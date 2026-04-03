use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use futures::StreamExt;
use tokio::{select, sync::mpsc};
use yazi_config::YAZI;
use yazi_shared::event::{Event, Replier};

pub(super) struct Signals {
	pub(super) tx: mpsc::UnboundedSender<(bool, Replier)>,
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
		use yazi_term::YIELD_TO_SUBPROCESS;

		match n {
			SIGINT => { /* ignored */ }
			SIGQUIT | SIGHUP | SIGTERM => {
				yazi_proxy::AppProxy::quit(Default::default());
				return false;
			}
			SIGTSTP => {
				tokio::spawn(async move {
					yazi_scheduler::AppProxy::stop().await;
					if unsafe { libc::kill(0, SIGSTOP) } != 0 {
						error!("Failed to stop the process:\n{}", std::io::Error::last_os_error());
						yazi_proxy::AppProxy::quit(Default::default());
					}
				});
			}
			SIGCONT if YIELD_TO_SUBPROCESS.try_acquire().is_ok() => {
				tokio::spawn(yazi_scheduler::AppProxy::resume());
			}
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
			CrosstermEvent::Mouse(mouse) if YAZI.mgr.mouse_events.get().contains(mouse.kind.into()) => {
				Event::Mouse(mouse).emit()
			}
			CrosstermEvent::Resize(..) => Event::Resize.emit(),
			CrosstermEvent::FocusGained => Event::Focus.emit(),
			CrosstermEvent::Paste(str) => Event::Paste(str).emit(),
			_ => {}
		}
	}

	fn spawn(mut rx: mpsc::UnboundedReceiver<(bool, Replier)>) -> Result<()> {
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
						Some((state, replier)) = rx.recv() => {
							term = term.filter(|_| state);
							replier.send(Ok(().into())).ok();
						},
						Some(n) = sys.next() => if !Self::handle_sys(n) { return },
						Some(Ok(e)) = t.next() => Self::handle_term(e)
					}
				} else {
					select! {
						biased;
						Some((state, replier)) = rx.recv() => {
							term = state.then(EventStream::new);
							replier.send(Ok(().into())).ok();
						},
						Some(n) = sys.next() => if !Self::handle_sys(n) { return },
					}
				}
			}
		});

		Ok(())
	}
}
