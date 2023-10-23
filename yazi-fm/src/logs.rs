use anyhow::{Context, Result};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt, Registry};
use yazi_config::BOOT;

pub(super) struct Logs;

impl Logs {
	pub(super) fn init() -> Result<WorkerGuard> {
		let appender = tracing_appender::rolling::never(&BOOT.state_dir, "yazi.log");
		let (handle, guard) = tracing_appender::non_blocking(appender);

		// let filter = EnvFilter::from_default_env();
		let subscriber = Registry::default().with(fmt::layer().pretty().with_writer(handle));

		tracing::subscriber::set_global_default(subscriber)
			.context("setting default subscriber failed")?;

		Ok(guard)
	}
}
