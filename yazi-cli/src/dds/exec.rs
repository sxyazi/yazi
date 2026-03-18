use std::str::FromStr;

use anyhow::{Result, bail};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use yazi_dds::{ID, Payload, Stream, ember::{Ember, EmberHi}};
use yazi_macro::try_format;
use yazi_shared::{Id, data::Data};

use crate::{CommandExec, CommandPub, dds::Dds};

impl Dds {
	pub(crate) async fn exec(cmd: CommandExec) -> anyhow::Result<Data> {
		let receiver = CommandPub::receiver()?;
		let req = cmd.body(*yazi_dds::ID)?;
		let resp = Self::ask("dds-exec", receiver, &req, "dds-exec-result").await?;

		#[derive(Deserialize)]
		struct Body {
			ok:    bool,
			#[serde(default)]
			value: Data,
			#[serde(default)]
			error: String,
		}

		let body = Body::deserialize(&resp)?;
		if body.ok {
			Ok(body.value)
		} else if !body.error.is_empty() {
			bail!("{}", body.error)
		} else {
			bail!("Unknown error")
		}
	}

	/// Send one custom message and wait for a matching custom reply.
	async fn ask(kind: &str, receiver: Id, body: &str, reply_kind: &str) -> Result<Data> {
		Ember::validate(kind)?;
		Ember::validate(reply_kind)?;

		let payload = try_format!(
			"{}\n{kind},{receiver},{ID},{body}\n",
			Payload::new(EmberHi::borrowed([reply_kind])),
		)?;

		let (mut lines, mut writer) = Stream::connect().await?;
		writer.write_all(payload.as_bytes()).await?;
		writer.flush().await?;
		drop(writer);

		while let Ok(Some(line)) = lines.next_line().await {
			match line.split(',').next() {
				Some("hey") => {
					if let Ok(Ember::Hey(hey)) = Payload::from_str(&line).map(|p| p.body) {
						Self::ensure_version(Some(&hey.version))?;
						Self::ensure_ability(&hey.peers, kind, receiver)?;
					}
				}
				Some(kind) if kind == reply_kind => match Payload::from_str(&line)?.body {
					Ember::Custom(body) => return Ok(body.data),
					_ => bail!("Expected custom payload of kind `{reply_kind}`"),
				},
				_ => {}
			}
		}

		bail!("Connection closed before receiving reply")
	}
}
