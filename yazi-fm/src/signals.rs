use anyhow::Result;
use tokio_stream::StreamExt;

pub(super) struct Signals;

impl Signals {
	pub(super) fn start() -> Result<()> {
		#[cfg(unix)]
		use rustix::process::Signal;

		#[cfg(unix)]
		let mut stream = signal_hook_tokio::Signals::new(
			[
				// Interrupt signals (Ctrl-C, Ctrl-\)
				Signal::INT,
				Signal::QUIT,
				// Hangup signal (Terminal closed)
				Signal::HUP,
				// Termination signal (kill)
				Signal::TERM,
				// Job control signals (Ctrl-Z, fg/bg)
				Signal::TSTP,
				Signal::CONT,
			]
			.map(Signal::as_raw),
		)?;
		#[cfg(windows)]
		let mut stream = tokio_stream::empty();

		tokio::spawn(async move {
			while let Some(n) = stream.next().await {
				if !Self::handle(n).await {
					break;
				}
			}
		});
		Ok(())
	}

	#[cfg(unix)]
	async fn handle(n: i32) -> bool {
		use rustix::process::{Signal, kill_current_process_group};
		use yazi_macro::error;
		use yazi_term::YIELD_TO_SUBPROCESS;

		let Some(signal) = Signal::from_named_raw(n) else {
			return true;
		};

		match signal {
			Signal::INT => { /* ignored */ }
			Signal::QUIT | Signal::HUP | Signal::TERM => {
				yazi_proxy::AppProxy::quit(Default::default());
				return false;
			}
			Signal::TSTP => {
				yazi_scheduler::AppProxy::stop().await;
				if let Err(e) = kill_current_process_group(Signal::STOP) {
					error!("Failed to stop the process:\n{e}");
					yazi_proxy::AppProxy::quit(Default::default());
				}
			}
			Signal::CONT if YIELD_TO_SUBPROCESS.try_acquire().is_ok() => {
				yazi_scheduler::AppProxy::resume().await;
			}
			_ => {}
		}
		true
	}

	#[cfg(windows)]
	async fn handle(_: ()) -> bool { unreachable!() }
}
