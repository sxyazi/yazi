use ui::App;

mod config;
mod core;
mod misc;
mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	// console_subscriber::init();

	config::init();

	core::adaptor::Adaptor::init();

	App::run().await
}
