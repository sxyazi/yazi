use std::str::FromStr;

use anyhow::{Result, bail};
use hashbrown::HashMap;
use tokio::io::AsyncWriteExt;
use yazi_dds::{ID, Payload, Peer, Stream, ember::{Ember, EmberBye, EmberHi}};
use yazi_macro::try_format;
use yazi_shared::Id;

use crate::dds::Dds;

impl Dds {
	/// Connect to an existing server to send a single message.
	pub(crate) async fn shot(kind: &str, receiver: Id, body: &str) -> Result<()> {
		Ember::validate(kind)?;

		let payload = try_format!(
			"{}\n{kind},{receiver},{ID},{body}\n{}\n",
			Payload::new(EmberHi::borrowed([])),
			Payload::new(EmberBye::borrowed())
		)?;

		let (mut lines, mut writer) = Stream::connect().await?;
		writer.write_all(payload.as_bytes()).await?;
		writer.flush().await?;
		drop(writer);

		let (mut peers, mut version) = Default::default();
		while let Ok(Some(line)) = lines.next_line().await {
			match line.split(',').next() {
				Some("hey") => {
					if let Ok(Ember::Hey(hey)) = Payload::from_str(&line).map(|p| p.body) {
						(peers, version) = (hey.peers, Some(hey.version));
					}
				}
				Some("bye") => break,
				_ => {}
			}
		}

		Self::ensure_version(version.as_deref())?;
		Self::ensure_ability(&peers, kind, receiver)?;
		Ok(())
	}

	pub(super) fn ensure_version(version: Option<&str>) -> Result<()> {
		if version != Some(EmberHi::version()) {
			bail!(
				"Incompatible version (Ya {}, Yazi {}). Restart all `ya` and `yazi` processes if you upgrade either one.",
				EmberHi::version(),
				version.unwrap_or("Unknown")
			);
		}
		Ok(())
	}

	pub(super) fn ensure_ability(peers: &HashMap<Id, Peer>, kind: &str, receiver: Id) -> Result<()> {
		match (receiver, peers.get(&receiver).map(|p| p.able(kind))) {
			// Send to all receivers
			(Id::ZERO, _) if peers.is_empty() => {
				bail!("No receiver found. Check if any receivers are running.")
			}
			(Id::ZERO, _) if peers.values().all(|p| !p.able(kind)) => {
				bail!("No receiver has the ability to receive `{kind}` messages.")
			}
			(Id::ZERO, _) => Ok(()),

			// Send to a specific receiver
			(_, Some(true)) => Ok(()),
			(_, Some(false)) => {
				bail!("Receiver `{receiver}` does not have the ability to receive `{kind}` messages.")
			}
			(_, None) => bail!("Receiver `{receiver}` not found. Check if the receiver is running."),
		}
	}
}
