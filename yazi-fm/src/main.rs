#![allow(clippy::module_inception)]

mod app;
mod completion;
mod executor;
mod help;
mod input;
mod logs;
mod panic;
mod root;
mod select;
mod signals;
mod tasks;
mod which;
mod widgets;

use executor::*;
use logs::*;
use panic::*;
#[cfg(feature = "plugin")]
use root::*;
use signals::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	_ = fdlimit::raise_fd_limit();
	// console_subscriber::init();

	yazi_config::init();

	yazi_core::init();

	#[cfg(feature = "plugin")]
	yazi_plugin::init();

	yazi_adaptor::init();

	app::App::run().await
}
