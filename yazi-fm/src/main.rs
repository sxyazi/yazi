#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

extern crate self as yazi_fm;

yazi_macro::mod_pub!(app cmp confirm help input mgr notify pick spot tasks which);

yazi_macro::mod_flat!(completion dispatcher executor logs panic renderer root router signals);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	Completion::complete()?;

	let events = yazi_headless::init_common()?;

	yazi_tty::init();

	yazi_emulator::init();

	yazi_boot::init();

	let _logs = yazi_fm::init()?;

	yazi_ffi::run(yazi_fm::serve(events)).await
}

fn init() -> anyhow::Result<Logs> {
	let logs = Logs::start()?;

	Signals::start()?;

	Panic::install();

	Ok(logs)
}

async fn serve(events: yazi_shared::event::EventRx) -> anyhow::Result<()> {
	yazi_version::setup!();

	yazi_term::setup()?;

	let _deinit = yazi_emulator::setup()?;

	yazi_config::setup()?;

	yazi_dds::serve();

	yazi_plugin::setup().await?;

	app::App::serve(events).await
}
