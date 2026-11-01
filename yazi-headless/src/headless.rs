use tokio::sync::mpsc::UnboundedReceiver;
use yazi_shared::event::EventRx;

#[must_use = "Keep the receivers alive while using headless services"]
pub struct Headless {
	_events: EventRx,
	_dds:    UnboundedReceiver<String>,
}

pub async fn init() -> anyhow::Result<Headless> {
	let events = init_common()?;

	yazi_boot::init_default();

	yazi_config::try_init(true)?;

	let dds = yazi_dds::init();

	yazi_plugin::setup().await?;

	Ok(Headless { _events: events, _dds: dds })
}

pub fn init_common() -> anyhow::Result<EventRx> {
	yazi_shim::init()?;

	let events = yazi_shared::init();

	yazi_fs::init();

	yazi_vfs::init();

	yazi_runner::init();

	Ok(events)
}
