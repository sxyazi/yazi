mod args;
mod package;

use std::collections::HashSet;

use args::*;
use clap::Parser;
use yazi_dds::dds_peer::DDSPeer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	if std::env::args_os().nth(1).is_some_and(|s| s == "-V" || s == "--version") {
		println!(
			"Ya {} ({} {})",
			env!("CARGO_PKG_VERSION"),
			env!("VERGEN_GIT_SHA"),
			env!("VERGEN_BUILD_DATE")
		);
		return Ok(());
	}

	match Args::parse().command {
		Command::Pub(cmd) => {
			yazi_dds::init();
			if let Err(e) = yazi_dds::Client::shot(&cmd.kind, cmd.receiver()?, None, &cmd.body()?).await {
				eprintln!("Cannot send message: {e}");
				std::process::exit(1);
			}
		}
		Command::PubStatic(cmd) => {
			yazi_dds::init();
			if let Err(e) = yazi_dds::Client::shot(&cmd.kind, 0, Some(cmd.severity), &cmd.body()?).await {
				eprintln!("Cannot send message: {e}");
				std::process::exit(1);
			}
		}
		Command::Pack(cmd) => {
			package::init();
			if cmd.install {
				package::Package::install_from_config("plugin", false).await?;
				package::Package::install_from_config("flavor", false).await?;
			} else if cmd.list {
				package::Package::list_from_config("plugin").await?;
				package::Package::list_from_config("flavor").await?;
			} else if cmd.upgrade {
				package::Package::install_from_config("plugin", true).await?;
				package::Package::install_from_config("flavor", true).await?;
			} else if let Some(repo) = &cmd.add {
				package::Package::add_to_config(repo).await?;
			}
		}

		Command::Sub(cmd) => {
			yazi_dds::init();
			let kinds = cmd.kinds.split(',').map(|s| s.to_owned()).collect::<HashSet<_>>();

			yazi_boot::BOOT.init(yazi_boot::Boot::init_with(kinds.clone(), kinds.clone()));
			yazi_dds::Client::echo_events_to_stdout(DDSPeer::from(cmd.sender), kinds);

			tokio::signal::ctrl_c().await?;
		}

		Command::SubStatic(cmd) => {
			yazi_dds::init();
			let kinds = cmd.kinds.split(',').map(|s| s.to_owned()).collect::<HashSet<_>>();

			yazi_boot::BOOT.init(yazi_boot::Boot::init_with(kinds.clone(), kinds.clone()));
			yazi_dds::Client::echo_events_to_stdout(DDSPeer::All, kinds);

			tokio::signal::ctrl_c().await?;
		}
	}

	Ok(())
}
