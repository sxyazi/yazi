#![allow(clippy::module_inception)]

mod app;
mod context;
mod executor;
mod header;
mod input;
mod logs;
mod manager;
mod root;
mod select;
mod signals;
mod status;
mod tasks;
mod which;

use app::*;
use context::*;
use executor::*;
use logs::*;
use root::*;
use signals::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	// console_subscriber::init();

	config::init();

	core::init();

	adaptor::init();

	App::run().await
}
