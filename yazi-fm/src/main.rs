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
mod term;
mod which;

use context::*;
use executor::*;
use logs::*;
use panic::*;
#[allow(unused_imports)]
use root::*;
use router::*;
use signals::*;
use term::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	Panic::install();
	Logs::start();

	_ = fdlimit::raise_fd_limit();

	yazi_shared::init();

	yazi_config::init()?;

	yazi_adapter::init();

	yazi_boot::init();

	yazi_proxy::init();

	yazi_dds::init();

	yazi_plugin::init()?;

	yazi_core::init();

	yazi_dds::serve();
	app::App::serve().await
}
