mod args;

use args::*;
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	if std::env::args_os().any(|s| s == "-V" || s == "--version") {
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
			if let Err(e) = yazi_dds::Client::shot(&cmd.kind, cmd.receiver, None, &cmd.body()?).await {
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
	}

	Ok(())
}
