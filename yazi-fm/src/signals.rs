use anyhow::Result;
use tokio_stream::StreamExt;

pub(super) struct Signals;

impl Signals {
	pub(super) fn start() -> Result<()> {
		#[cfg(unix)]
		use libc::{SIGCONT, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGTSTP};

		#[cfg(unix)]
		let mut stream = signal_hook_tokio::Signals::new([
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
	async fn handle(n: libc::c_int) -> bool {
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
				yazi_scheduler::AppProxy::stop().await;
				if unsafe { libc::kill(0, SIGSTOP) } != 0 {
					error!("Failed to stop the process:\n{}", std::io::Error::last_os_error());
					yazi_proxy::AppProxy::quit(Default::default());
				}
			}
			SIGCONT if YIELD_TO_SUBPROCESS.try_acquire().is_ok() => {
				yazi_scheduler::AppProxy::resume().await;
			}
			_ => {}
		}
		true
	}

	#[cfg(windows)]
	async fn handle(_: ()) -> bool { unreachable!() }
}
