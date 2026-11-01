yazi_macro::mod_pub!(cache cmp dds env package shared);

yazi_macro::mod_flat!(args);

use clap::Parser;
use yazi_macro::{errln, outln};
use yazi_shim::anyhow::ErrorExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let _events = yazi_headless::init_common()?;

	let Err(e) = run().await else { return Ok(()) };
	if e.is_io_error(std::io::ErrorKind::BrokenPipe) { Ok(()) } else { Err(e) }
}

async fn run() -> anyhow::Result<()> {
	yazi_version::setup!();

	match Args::parse().command {
		Command::Emit(cmd) => {
			yazi_boot::init_default();
			let _dds = yazi_dds::init();
			if let Err(e) = dds::Dds::shot("dds-emit", CommandPub::receiver()?, &cmd.body()?).await {
				errln!("Cannot emit command: {e}")?;
				std::process::exit(1);
			}
		}

		Command::EmitTo(cmd) => {
			yazi_boot::init_default();
			let _dds = yazi_dds::init();
			if let Err(e) = dds::Dds::shot("dds-emit", cmd.receiver, &cmd.body()?).await {
				errln!("Cannot emit command: {e}")?;
				std::process::exit(1);
			}
		}

		Command::Exec(cmd) => {
			yazi_boot::init_default();
			let _dds = yazi_dds::init();

			match dds::Dds::exec(cmd).await {
				Ok(data) => outln!("{}", serde_json::to_string(&data)?)?,
				Err(e) => {
					errln!("Cannot execute command: {e}")?;
					std::process::exit(1);
				}
			}
		}

		Command::Pkg(cmd) => {
			package::init()?;

			let mut pkg = package::Package::load().await?;
			match cmd {
				CommandPkg::Add { ids } => pkg.add_many(&ids).await?,
				CommandPkg::Delete { ids, discard } => pkg.delete_many(&ids, discard).await?,
				CommandPkg::Install { discard } => pkg.install(discard).await?,
				CommandPkg::List => pkg.print()?,
				CommandPkg::Upgrade { ids, discard } => pkg.upgrade_many(&ids, discard).await?,
			}
		}

		Command::Pub(cmd) => {
			yazi_boot::init_default();
			let _dds = yazi_dds::init();
			if let Err(e) = dds::Dds::shot(&cmd.kind, CommandPub::receiver()?, &cmd.body()?).await {
				errln!("Cannot send message: {e}")?;
				std::process::exit(1);
			}
		}

		Command::PubTo(cmd) => {
			yazi_boot::init_default();
			let _dds = yazi_dds::init();
			if let Err(e) = dds::Dds::shot(&cmd.kind, cmd.receiver, &cmd.body()?).await {
				errln!("Cannot send message: {e}")?;
				std::process::exit(1);
			}
		}

		Command::Sub(cmd) => {
			yazi_boot::init_default();
			let _dds = yazi_dds::init();
			dds::Dds::draw(cmd.kinds.split(',').collect()).await?;

			tokio::signal::ctrl_c().await?;
		}

		Command::Cache(cmd) => {
			yazi_tty::init();
			yazi_config::setup()?;

			match cmd {
				CommandCache::Clear => {
					cache::Cache::clear()?;
				}
			}
		}

		Command::Env => {
			yazi_tty::init();
			yazi_term::setup()?;
			yazi_config::setup()?;
			outln!("{}", env::Env::print().await?)?;
		}

		Command::Cmp(cmp) => cmp.complete()?,
	}

	Ok(())
}
