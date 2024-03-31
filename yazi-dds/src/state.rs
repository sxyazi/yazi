use std::{collections::HashMap, mem, ops::Deref, sync::atomic::{AtomicU64, Ordering}, time::UNIX_EPOCH};

use anyhow::Result;
use parking_lot::RwLock;
use tokio::{fs::{self, File, OpenOptions}, io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter}};
use yazi_boot::BOOT;
use yazi_shared::{timestamp_us, RoCell};

use crate::{body::Body, CLIENTS};

pub static STATE: RoCell<State> = RoCell::new();

#[derive(Default)]
pub struct State {
	inner: RwLock<Option<HashMap<String, String>>>,
	last:  AtomicU64,
}

impl Deref for State {
	type Target = RwLock<Option<HashMap<String, String>>>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl State {
	pub fn add(&self, key: String, value: &str) {
		if let Some(ref mut inner) = *self.inner.write() {
			inner.insert(key, value.to_owned());
			self.last.store(timestamp_us(), Ordering::Relaxed);
		}
	}

	pub async fn load(&self) -> Result<()> {
		let mut buf = BufReader::new(File::open(BOOT.state_dir.join(".dds")).await?);
		let mut line = String::new();

		let mut inner = HashMap::new();
		while buf.read_line(&mut line).await? > 0 {
			let mut parts = line.splitn(5, ',');
			let Some(kind) = parts.next() else { continue };
			let Some(_) = parts.next() else { continue };
			let Some(severity) = parts.next().and_then(|s| s.parse::<u8>().ok()) else { continue };
			let Some(_) = parts.next() else { continue };
			let Some(body) = parts.next() else { continue };
			inner.insert(format!("{}_{severity}_{kind}", Body::tab(kind, body)), mem::take(&mut line));
		}

		let clients = CLIENTS.read();
		for payload in inner.values() {
			clients.values().for_each(|c| _ = c.tx.send(format!("{payload}\n")));
		}

		self.inner.write().replace(inner);
		self.last.store(timestamp_us(), Ordering::Relaxed);
		Ok(())
	}

	pub async fn drain(&self) -> Result<()> {
		let Some(inner) = self.inner.write().take() else { return Ok(()) };
		if self.skip().await.unwrap_or(false) {
			return Ok(());
		}

		let mut buf = BufWriter::new(
			OpenOptions::new()
				.write(true)
				.create(true)
				.truncate(true)
				.open(BOOT.state_dir.join(".dds"))
				.await?,
		);

		let mut state = inner.into_iter().collect::<Vec<_>>();
		state.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
		for (_, v) in state {
			buf.write_all(v.as_bytes()).await?;
			buf.write_u8(b'\n').await?;
		}

		Ok(())
	}

	async fn skip(&self) -> Result<bool> {
		let meta = fs::symlink_metadata(BOOT.state_dir.join(".dds")).await?;
		let modified = meta.modified()?.duration_since(UNIX_EPOCH)?.as_micros();
		Ok(modified >= self.last.load(Ordering::Relaxed) as u128)
	}
}
