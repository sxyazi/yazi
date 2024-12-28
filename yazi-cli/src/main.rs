yazi_macro::mod_pub!(package);

yazi_macro::mod_flat!(args);

use std::process::ExitCode;

use clap::Parser;

#[tokio::main]
async fn main() -> ExitCode {
	match run().await {
		Ok(()) => ExitCode::SUCCESS,
		Err(err) => {
			// Look for a broken pipe error. In this case, we generally want
			// to exit "gracefully" with a success exit code. This matches
			// existing Unix convention. We need to handle this explicitly
			// since the Rust runtime doesn't ask for PIPE signals, and thus
			// we get an I/O error instead. Traditional C Unix applications
			// quit by getting a PIPE signal that they don't handle, and thus
			// the unhandled signal causes the process to unceremoniously
			// terminate.
			for cause in err.chain() {
				if let Some(ioerr) = cause.downcast_ref::<std::io::Error>() {
					if ioerr.kind() == std::io::ErrorKind::BrokenPipe {
						return ExitCode::from(0);
					}
				}
			}
			eprintln!("{:#}", err);
			ExitCode::FAILURE
		}
	}
}

async fn run() -> anyhow::Result<()> {
	yazi_shared::init();
	yazi_fs::init();

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
		Command::Emit(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();
			if let Err(e) =
				yazi_dds::Client::shot("dds-emit", CommandPub::receiver()?, &cmd.body()?).await
			{
				eprintln!("Cannot emit command: {e}");
				std::process::exit(1);
			}
		}

		Command::EmitTo(cmd) => {
			yazi_boot::init_default();
			yazi_dds::init();
			if let Err(e) = yazi_dds::Client::shot("dds-emit", cmd.receiver, &cmd.body()?).await {
				eprintln!("Cannot emit command: {e}");
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
	}

	Ok(())
}
