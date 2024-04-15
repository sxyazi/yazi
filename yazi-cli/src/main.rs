mod args;

use args::*;
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let args = Args::parse();

	match &args.command {
		Command::Send(cmd) => {
			yazi_dds::init();
			if let Err(e) = yazi_dds::Client::shot(&cmd.message).await {
				eprintln!("Cannot send message: {e}");
				std::process::exit(1);
			}
		}
	}

	Ok(())
}
