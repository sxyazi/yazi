yazi_macro::mod_pub!(package);

yazi_macro::mod_flat!(args);

use std::process::ExitCode;

use clap::Parser;
use yazi_macro::{errln, outln};

#[tokio::main]
async fn main() -> ExitCode {
	match run().await {
		Ok(()) => ExitCode::SUCCESS,
		Err(e) => {
			for cause in e.chain() {
				if let Some(ioerr) = cause.downcast_ref::<std::io::Error>() {
					if ioerr.kind() == std::io::ErrorKind::BrokenPipe {
						return ExitCode::from(0);
					}
				}
			}
			errln!("{e:#}").ok();
			ExitCode::FAILURE
		}
	}
}

async fn run() -> anyhow::Result<()> {
	yazi_shared::init();
	yazi_fs::init();

	if std::env::args_os().nth(1).is_some_and(|s| s == "-V" || s == "--version") {
		outln!(
			"Ya {} ({} {})",
			env!("CARGO_PKG_VERSION"),
			env!("VERGEN_GIT_SHA"),
			env!("VERGEN_BUILD_DATE")
		)?;
		return Ok(());
	}

	match Args::parse().command {
		Command::Emit(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();
			if let Err(e) =
				yazi_dds::Client::shot("dds-emit", CommandPub::receiver()?, &cmd.body()?).await
			{
				errln!("Cannot emit command: {e}")?;
				std::process::exit(1);
			}
		}

		Command::EmitTo(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();
			if let Err(e) = yazi_dds::Client::shot("dds-emit", cmd.receiver, &cmd.body()?).await {
				errln!("Cannot emit command: {e}")?;
				std::process::exit(1);
			}
		}

		Command::Pkg(cmd) => {
			package::init()?;

			let mut pkg = package::Package::load().await?;
			match cmd {
				CommandPkg::Add { ids } => pkg.add_many(&ids).await?,
				CommandPkg::Delete { ids } => pkg.delete_many(&ids).await?,
				CommandPkg::Install => pkg.install(false).await?,
				CommandPkg::List => pkg.print()?,
				CommandPkg::Upgrade => pkg.install(true).await?,
			}
		}

		Command::Pack(cmd) => {
			package::init()?;
			outln!(
				"WARNING: `ya pack` is deprecated, use the new `ya pkg` instead. See https://github.com/sxyazi/yazi/pull/2770 for more details."
			)?;
			if cmd.install {
				package::Package::load().await?.install(false).await?;
			} else if cmd.list {
				package::Package::load().await?.print()?;
			} else if cmd.upgrade {
				package::Package::load().await?.install(true).await?;
			} else if let Some(uses) = cmd.add {
				package::Package::load().await?.add_many(&uses).await?;
			} else if let Some(uses) = cmd.delete {
				package::Package::load().await?.delete_many(&uses).await?;
			}
		}

		Command::Pub(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();
			if let Err(e) = yazi_dds::Client::shot(&cmd.kind, CommandPub::receiver()?, &cmd.body()?).await
			{
				errln!("Cannot send message: {e}")?;
				std::process::exit(1);
			}
		}

		Command::PubTo(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();
			if let Err(e) = yazi_dds::Client::shot(&cmd.kind, cmd.receiver, &cmd.body()?).await {
				errln!("Cannot send message: {e}")?;
				std::process::exit(1);
			}
		}

		Command::Sub(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();
			yazi_dds::Client::draw(cmd.kinds.split(',').collect()).await?;

			tokio::signal::ctrl_c().await?;
		}
	}

	Ok(())
}
