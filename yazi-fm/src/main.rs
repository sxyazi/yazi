#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

extern crate self as yazi_fm;

yazi_macro::mod_pub!(app cmp confirm help input mgr notify pick spot tasks which);

yazi_macro::mod_flat!(dispatcher executor logs panic renderer root router signals);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	yazi_shim::init()?;

	yazi_shared::init();

	yazi_tty::init();

	yazi_emulator::init();

	yazi_fs::init();

	yazi_vfs::init();

	yazi_runner::init(yazi_plugin::slim_lua);

	yazi_adapter::init();

	yazi_widgets::init();

	yazi_watcher::init();

	yazi_actor::init();

	yazi_fm::init();

	settle(yazi_fm::serve().await)
}

fn init() {
	if yazi_version::has_dash_v() {
		println!("Yazi\n{}", yazi_version::version_full());
		std::process::exit(0);
	}
}

async fn serve() -> anyhow::Result<()> {
	Logs::start()?;
	Signals::start()?;

	yazi_term::setup()?;

	let _deinit = yazi_emulator::setup()?;
	Panic::install();

	yazi_config::setup()?;

	yazi_boot::setup()?;

	yazi_dds::serve();

	yazi_plugin::setup()?;

	yazi_shared::LOCAL_SET.run_until(app::App::serve()).await
}

fn settle(result: anyhow::Result<()>) -> anyhow::Result<()> {
	match &result {
		Err(e) if let Some(e) = e.downcast_ref::<clap::Error>() => e.exit(),
		_ => result,
	}
}
