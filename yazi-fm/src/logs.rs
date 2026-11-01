use std::{fs::File, io};

use anyhow::Context;
use ratatui_core::style::Color;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use yazi_fs::Xdg;
use yazi_macro::writef;
use yazi_shim::log::LOG_LEVEL;
use yazi_tty::sequence::{SetFg, SetSgr};

#[must_use = "Keep the guard alive to ensure logs are flushed"]
pub(super) struct Logs {
	_guard: Option<WorkerGuard>,
}

impl Logs {
	pub(super) fn start() -> anyhow::Result<Self> {
		let level = LOG_LEVEL.get();
		if level.is_none() {
			return Ok(Self { _guard: None });
		}

		let state_dir = Xdg::state_dir();
		std::fs::create_dir_all(state_dir)
			.with_context(|| format!("failed to create state directory: {state_dir:?}"))?;

		let log_path = state_dir.join("yazi.log");
		let log_file = File::create(&log_path)
			.with_context(|| format!("failed to create log file: {log_path:?}"))?;

		let (non_blocking, guard) = tracing_appender::non_blocking(log_file);
		tracing_subscriber::fmt()
			.pretty()
			.with_env_filter(EnvFilter::default().add_directive(level.as_ref().parse()?))
			.with_writer(non_blocking)
			.with_ansi(cfg!(debug_assertions))
			.init();

		writef!(
			io::stderr(),
			"{}Running with log level `{level}`, logs are written to {log_path:?}\n{}",
			SetFg(Color::Yellow),
			SetSgr::Reset,
		)?;

		Ok(Self { _guard: Some(guard) })
	}
}
