mod args;
mod package;

use args::*;
use clap::Parser;

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
			yazi_boot::init_default();
			yazi_dds::init();
			if let Err(e) = yazi_dds::Client::shot(&cmd.kind, cmd.receiver()?, &cmd.body()?).await {
				eprintln!("Cannot send message: {e}");
				std::process::exit(1);
			}
		}

		Command::PubTo(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();
			if let Err(e) = yazi_dds::Client::shot(&cmd.kind, cmd.receiver, &cmd.body()?).await {
				eprintln!("Cannot send message: {e}");
				std::process::exit(1);
			}
		}

		Command::Sub(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();
			yazi_dds::Client::draw(cmd.kinds.split(',').collect()).await?;

			tokio::signal::ctrl_c().await?;
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
	}

	Ok(())
}
