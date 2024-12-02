use std::{env, fs::File};

use anyhow::Context;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use yazi_fs::Xdg;
use yazi_shared::RoCell;

static _GUARD: RoCell<WorkerGuard> = RoCell::new();

pub(super) struct Logs;

impl Logs {
	pub(super) fn start() -> anyhow::Result<()> {
		let mut level = env::var("YAZI_LOG").unwrap_or_default();
		level.make_ascii_uppercase();
		if !matches!(level.as_str(), "ERROR" | "WARN" | "INFO" | "DEBUG") {
			return Ok(());
		}

		let state_dir = Xdg::state_dir();
		std::fs::create_dir_all(&state_dir)
			.with_context(|| format!("failed to create state directory: {state_dir:?}"))?;

		let log_path = state_dir.join("yazi.log");
		let log_file = File::create(&log_path)
			.with_context(|| format!("failed to create log file: {log_path:?}"))?;

		let (non_blocking, guard) = tracing_appender::non_blocking(log_file);
		tracing_subscriber::fmt()
			.pretty()
			.with_env_filter(EnvFilter::new(&level))
			.with_writer(non_blocking)
			.with_ansi(cfg!(debug_assertions))
			.init();

		_GUARD.init(guard);
		eprintln!("Running with log level `{level}`, logs are written to {log_path:?}");

		Ok(())
	}
}
