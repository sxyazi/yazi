#![allow(clippy::if_same_then_else, clippy::module_inception, clippy::unit_arg)]

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

yazi_macro::mod_pub!(app cmp confirm help input lives mgr notify pick spot tasks which);

yazi_macro::mod_flat!(context executor logs panic root router signals term);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	Panic::install();
	yazi_shared::init();

	Logs::start()?;
	_ = fdlimit::raise_fd_limit();

	yazi_term::init();

	yazi_fs::init();

	yazi_config::init()?;

	yazi_adapter::init()?;

	yazi_boot::init();

	yazi_proxy::init();

	yazi_dds::init();

	yazi_plugin::init()?;

	yazi_core::init();

	yazi_dds::serve();
	app::App::serve().await
}
