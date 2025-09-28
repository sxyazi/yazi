use std::fs::File;

use anyhow::Context;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use yazi_fs::Xdg;
use yazi_shared::{LOG_LEVEL, RoCell};

static _GUARD: RoCell<WorkerGuard> = RoCell::new();

pub(super) struct Logs;

impl Logs {
	pub(super) fn start() -> anyhow::Result<()> {
		let level = LOG_LEVEL.get();
		if level.is_none() {
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
			.with_env_filter(EnvFilter::new(level))
			.with_writer(non_blocking)
			.with_ansi(cfg!(debug_assertions))
			.init();

		_GUARD.init(guard);
		Ok(crossterm::execute!(
			std::io::stderr(),
			SetForegroundColor(Color::Yellow),
			Print(format!("Running with log level `{level}`, logs are written to {log_path:?}\n")),
			ResetColor
		)?)
	}
}
