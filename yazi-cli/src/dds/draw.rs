use anyhow::{Context, Result};
use hashbrown::HashSet;
use tokio::{io::AsyncWriteExt, time};
use yazi_dds::{ClientReader, Payload, Stream, ember::EmberHi};
use yazi_macro::try_format;

use crate::dds::Dds;

impl Dds {
	/// Connect to an existing server and listen in on the messages that are being
	/// sent by other yazi instances:
	///   - If no server is running, fail right away;
	///   - If a server is closed, attempt to reconnect forever.
	pub(crate) async fn draw(kinds: HashSet<&str>) -> Result<()> {
		async fn make(kinds: &HashSet<&str>) -> Result<ClientReader> {
			let (lines, mut writer) = Stream::connect().await?;
			let hi = Payload::new(EmberHi::borrowed(kinds.iter().copied()));
			writer.write_all(try_format!("{hi}\n")?.as_bytes()).await?;
			writer.flush().await?;
			Ok(lines)
		}

		let mut lines = make(&kinds).await.context("No running Yazi instance found")?;
		loop {
			match lines.next_line().await? {
				Some(s) => {
					let kind = s.split(',').next();
					if matches!(kind, Some(kind) if kinds.contains(kind)) {
						println!("{s}");
					}
				}
				None => loop {
					time::sleep(time::Duration::from_secs(1)).await;
					if let Ok(new) = make(&kinds).await {
						lines = new;
						break;
					}
				},
			}
		}
	}
}
