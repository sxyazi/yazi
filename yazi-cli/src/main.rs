mod args;

use args::*;
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let args = Args::parse();

	match &args.command {
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
