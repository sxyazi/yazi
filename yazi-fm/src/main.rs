#![allow(clippy::module_inception)]

mod app;
mod executor;
mod help;
mod input;
mod logs;
mod root;
mod select;
mod signals;
mod tasks;
mod which;

use app::*;
use executor::*;
use logs::*;
use root::*;
use signals::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	// console_subscriber::init();

	yazi_config::init();

	yazi_core::init();

	yazi_plugin::init();

	yazi_adaptor::init();

	App::run().await
}
