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
			errln!("{:#}", e).ok();
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

		Command::Pack(cmd) => {
			package::init()?;
			if cmd.install {
				package::Package::install_from_config("plugin", false).await?;
				package::Package::install_from_config("flavor", false).await?;
			} else if cmd.list {
				package::Package::list_from_config("plugin").await?;
				package::Package::list_from_config("flavor").await?;
			} else if cmd.upgrade {
				package::Package::install_from_config("plugin", true).await?;
				package::Package::install_from_config("flavor", true).await?;
			} else if let Some(repo) = cmd.add {
				package::Package::add_to_config(&repo).await?;
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
