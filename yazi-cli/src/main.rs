mod args;

use std::process;

use args::*;
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let args = Args::parse();

	if args.version {
		println!("Ya {}", action_version());
		process::exit(0);
	}

	match &args.command.expect("No command provided") {
		Command::Pub(cmd) => {
			yazi_dds::init();
			if let Err(e) = yazi_dds::Client::shot(&cmd.kind, cmd.receiver, None, &cmd.body()?).await {
				eprintln!("Cannot send message: {e}");
				process::exit(1);
			}
		}
		Command::PubStatic(cmd) => {
			yazi_dds::init();
			if let Err(e) = yazi_dds::Client::shot(&cmd.kind, 0, Some(cmd.severity), &cmd.body()?).await {
				eprintln!("Cannot send message: {e}");
				process::exit(1);
			}
		}
	}

	Ok(())
}

fn action_version() -> String {
	format!(
		"{} ({} {})",
		env!("CARGO_PKG_VERSION"),
		env!("VERGEN_GIT_SHA"),
		env!("VERGEN_BUILD_DATE")
	)
}
