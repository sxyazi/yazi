#![allow(clippy::module_inception)]

mod app;
mod executor;
mod header;
mod help;
mod input;
mod logs;
mod manager;
mod parser;
mod root;
mod select;
mod signals;
mod status;
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

	config::init();

	core::init();

	plugin::init();

	adaptor::init();

	App::run().await
}
