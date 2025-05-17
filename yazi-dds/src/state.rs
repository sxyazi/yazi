use std::{collections::HashMap, mem, ops::Deref, sync::atomic::{AtomicU64, Ordering}, time::UNIX_EPOCH};

use anyhow::Result;
use parking_lot::RwLock;
use tokio::{fs::{self, File, OpenOptions}, io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter}};
use yazi_boot::BOOT;
use yazi_shared::{RoCell, timestamp_us};

use crate::CLIENTS;

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
	pub fn set(&self, kind: &str, sender: u64, body: &str) -> bool {
		let Some(inner) = &mut *self.inner.write() else { return false };

		if body == "null" {
			return inner
				.remove(kind)
				.map(|_| self.last.store(timestamp_us(), Ordering::Relaxed))
				.is_some();
		}

		let value = format!("{kind},0,{sender},{body}\n");
		if inner.get(kind).is_some_and(|s| *s == value) {
			return false;
		}

		inner.insert(kind.to_owned(), value);
		self.last.store(timestamp_us(), Ordering::Relaxed);
		true
	}

	pub async fn load_or_create(&self) {
		if self.load().await.is_err() {
			self.inner.write().replace(Default::default());
			self.last.store(timestamp_us(), Ordering::Relaxed);
		}
	}

	pub async fn drain(&self) -> Result<()> {
		let Some(inner) = self.inner.write().take() else { return Ok(()) };
		if self.skip().await.unwrap_or(false) {
			return Ok(());
		}

		fs::create_dir_all(&BOOT.state_dir).await?;
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
		}

		buf.flush().await?;
		Ok(())
	}

	async fn load(&self) -> Result<()> {
		let mut file = BufReader::new(File::open(BOOT.state_dir.join(".dds")).await?);
		let mut buf = String::new();

		let mut inner = HashMap::new();
		while file.read_line(&mut buf).await? > 0 {
			let line = mem::take(&mut buf);
			let mut parts = line.splitn(4, ',');

			let Some(kind) = parts.next() else { continue };
			let Some(_) = parts.next() else { continue };
			inner.insert(kind.to_owned(), line);
		}

		let clients = CLIENTS.read();
		for payload in inner.values() {
			clients.values().for_each(|c| _ = c.tx.send(payload.clone()));
		}

		self.inner.write().replace(inner);
		self.last.store(timestamp_us(), Ordering::Relaxed);
		Ok(())
	}

	async fn skip(&self) -> Result<bool> {
		let meta = fs::symlink_metadata(BOOT.state_dir.join(".dds")).await?;
		let modified = meta.modified()?.duration_since(UNIX_EPOCH)?.as_micros();
		Ok(modified >= self.last.load(Ordering::Relaxed) as u128)
	}
}
