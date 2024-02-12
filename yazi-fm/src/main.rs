#![allow(clippy::module_inception)]
#![allow(clippy::unit_arg)]

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod app;
mod completion;
mod components;
mod context;
mod executor;
mod help;
mod input;
mod lives;
mod logs;
mod notify;
mod panic;
mod root;
mod router;
mod select;
mod signals;
mod tasks;
mod which;
mod widgets;

use context::*;
use executor::*;
use logs::*;
use panic::*;
#[allow(unused_imports)]
use root::*;
use router::*;
use signals::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	_ = fdlimit::raise_fd_limit();
	// console_subscriber::init();

	yazi_config::init();

	yazi_core::init();

	yazi_scheduler::init();

	yazi_plugin::init();

	yazi_adaptor::init();

	app::App::run().await
}
