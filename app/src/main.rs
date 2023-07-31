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

pub(self) use app::*;
pub(self) use context::*;
pub(self) use executor::*;
pub(self) use logs::*;
pub(self) use root::*;
pub(self) use signals::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	// console_subscriber::init();

	config::init();

	core::adaptor::Adaptor::init();

	App::run().await
}
